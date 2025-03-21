use crate::models::*;
use bollard::{
    Docker,
    container::{ListContainersOptions, StatsOptions},
};
use futures::StreamExt;
use log::{debug, trace, warn};
use sysinfo::{Disks, Networks, System};

pub fn collect_general_info(sys: &System) -> GeneralInfo {
    debug!("Collecting general system information");
    // CPU info
    let cores_usage: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    let average_usage = cores_usage.iter().sum::<f32>() / cores_usage.len() as f32;
    let cpu_info = CpuInfo {
        count: sys.cpus().len(),
        avg_usage: average_usage,
        usage: cores_usage,
    };
    trace!(
        "CPU info collected: {} cores, {:.2}% avg usage",
        cpu_info.count, cpu_info.avg_usage
    );

    // Memory info
    let memory_info = MemoryInfo {
        total_mem: sys.total_memory(),
        used_mem: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
    };

    // System load info
    let system_info = SystemInfo {
        name: System::name().unwrap_or_default(),
        kernel_ver: System::kernel_version().unwrap_or_default(),
        os_ver: System::os_version().unwrap_or_default(),
        os_name: System::long_os_version().unwrap_or_default(),
        host_name: System::host_name().unwrap_or_default(),
        load_avg: vec![
            System::load_average().one,
            System::load_average().five,
            System::load_average().fifteen,
        ],
        uptime: System::uptime(),
    };

    // Network info
    let networks = Networks::new_with_refreshed_list();
    let interfaces = networks
        .iter()
        .map(|(name, data)| NetworkInterface {
            name: name.to_string(),
            rx: data.total_received(),
            tx: data.total_transmitted(),
        })
        .collect();
    let network_info = NetworkInfo { interfaces };

    // Disk info
    let disks = Disks::new_with_refreshed_list();
    trace!("Disks: {:?}", disks);

    let disk_info = DisksInfo {
        disks: disks
            .iter()
            .filter(|disk| {
                let fs = disk.file_system().to_str().unwrap_or_default();
                let mount_point = disk.mount_point().to_str().unwrap_or_default();
                // Skip non-filesystems and system partitions
                if fs.is_empty()
                    || mount_point.starts_with("/sys")
                    || mount_point.starts_with("/proc")
                    || mount_point.starts_with("/etc")
                    || mount_point.starts_with("/app")
                {
                    return false;
                }
                // Common filesystem types
                matches!(
                    fs.to_lowercase().as_str(),
                    "ext2"
                        | "ext3"
                        | "ext4"
                        | "btrfs"
                        | "xfs"
                        | "zfs"
                        | "ntfs"
                        | "fat"
                        | "fat32"
                        | "exfat"
                        | "hfs"
                        | "hfs+"
                        | "apfs"
                        | "jfs"
                        | "reiserfs"
                        | "ufs"
                        | "f2fs"
                        | "nilfs2"
                        | "hpfs"
                        | "minix"
                        | "qnx4"
                        | "ocfs2"
                        | "udf"
                        | "vfat"
                        | "msdos"
                )
            })
            .map(|disk| DiskInfo {
                fs: disk.file_system().to_str().unwrap_or_default().to_string(),
                kind: disk.kind().to_string(),
                total_space: disk.total_space(),
                free_space: disk.available_space(),
                mount_point: disk.mount_point().to_str().unwrap_or_default().to_string(),
                removable: disk.is_removable(),
                io: [
                    disk.usage().read_bytes,
                    disk.usage().written_bytes,
                    disk.usage().total_read_bytes,
                    disk.usage().total_written_bytes,
                ],
            })
            .collect(),
    };

    debug!("General system information collection completed");
    GeneralInfo {
        t: chrono::Utc::now().timestamp(),
        sys: system_info,
        mem: memory_info,
        cpu: cpu_info,
        net: network_info,
        disk: disk_info,
    }
}

pub fn collect_processes_info(sys: &System) -> ProcessesInfo {
    debug!("Collecting processes information");
    let processes = sys
        .processes()
        .values()
        .map(|process| ProcessInfo {
            pid: process.pid().as_u32(),
            name: process.name().to_str().unwrap_or_default().to_string(),
            runtime: process.run_time(),
            cpu: process.cpu_usage(),
            mem: process.memory(),
            stat: process.status().to_string(),
            cmd: process
                .cmd()
                .iter()
                .map(|x| x.to_str().unwrap_or_default())
                .collect::<Vec<&str>>()
                .join(" "),
            env: process
                .environ()
                .iter()
                .map(|x| x.to_str().unwrap_or_default())
                .collect::<Vec<&str>>()
                .join("; "),
        })
        .collect();

    debug!(
        "Collected information for {} processes",
        sys.processes().len()
    );
    ProcessesInfo {
        t: chrono::Utc::now().timestamp(),
        processes,
    }
}

pub async fn get_docker_containers() -> Option<DockerInfo> {
    debug!("Attempting to connect to Docker daemon");
    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) => {
            debug!("Successfully connected to Docker daemon");
            docker
        }
        Err(e) => {
            warn!("Failed to connect to Docker daemon: {}", e);
            return None;
        }
    };

    // List all containers
    debug!("Listing Docker containers");
    let containers = match docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
    {
        Ok(containers) => {
            debug!("Found {} Docker containers", containers.len());
            containers
        }
        Err(e) => {
            warn!("Failed to list Docker containers: {}", e);
            return None;
        }
    };

    let mut result = Vec::new();

    // Pre-collect all stats futures
    debug!("Gathering stats for {} containers", containers.len());
    let stats_futures = containers
        .iter()
        .map(|container| {
            let container_id = container.id.clone().unwrap_or_default();
            trace!("Requesting stats for container {}", container_id);
            let mut stats_stream = docker.stats(
                &container_id,
                Some(StatsOptions {
                    stream: false,
                    ..Default::default()
                }),
            );
            async move { (container.clone(), stats_stream.next().await) }
        })
        .collect::<Vec<_>>();

    // Resolve all futures in parallel
    let results = futures::future::join_all(stats_futures).await;

    for (container, stats_result) in results {
        let container_id = container.id.clone().unwrap_or_default();

        // Get container stats
        let stats = match stats_result {
            Some(Ok(stats)) => stats,
            Some(Err(e)) => {
                warn!("Failed to get stats for container {}: {}", container_id, e);
                continue;
            }
            None => {
                warn!("No stats available for container {}", container_id);
                continue;
            }
        };

        trace!("Processing stats for container {}", container_id);

        // Calculate CPU usage
        // https://github.com/moby/moby/blob/eb131c5383db8cac633919f82abad86c99bffbe5/cli/command/container/stats_helpers.go#L175-L188
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
            - stats.precpu_stats.cpu_usage.total_usage as f64;
        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        let cpu_usage = if system_delta > 0.0 && cpu_delta > 0.0 {
            (cpu_delta / system_delta)
                * stats
                    .cpu_stats
                    .cpu_usage
                    .percpu_usage
                    .unwrap_or_default()
                    .len() as f64
        } else {
            0.0
        };

        // Parse ports
        let ports = container
            .ports
            .unwrap_or_default()
            .iter()
            .map(|port| DockerPort {
                ip: port.ip.clone(),
                priv_port: port.private_port,
                pub_port: port.public_port,
                protocol: port.typ.clone().unwrap().to_string(),
            })
            .collect();

        // Create container info
        result.push(DockerContainer {
            id: container_id.clone(),
            name: container.names.unwrap_or_default().join(", "),
            image: container.image.unwrap_or_default(),
            status: container.status.unwrap_or_default(),
            state: container.state.unwrap_or_default(),
            created: container.created.unwrap_or(0),
            ports,
            cpu_usage,
            mem_usage: stats.memory_stats.usage.unwrap_or(0),
            mem_limit: stats.memory_stats.limit.unwrap_or(0),
            net_io: match stats.network {
                Some(network) => [network.rx_bytes, network.tx_bytes],
                None => {
                    trace!("No network stats for container {}", container_id);
                    [0, 0]
                }
            },
            disk_io: [
                stats.storage_stats.read_size_bytes.unwrap_or(0),
                stats.storage_stats.write_size_bytes.unwrap_or(0),
            ],
        });
    }

    debug!(
        "Successfully collected data for {} Docker containers",
        result.len()
    );
    Some(DockerInfo {
        t: chrono::Utc::now().timestamp(),
        containers: result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gather_data() {
        use std::time::Instant;
        let mut now = Instant::now();
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_all();
        println!("Elapsed: {:.2?}", now.elapsed());

        now = Instant::now();
        println!("{}", serde_json::json!(collect_general_info(&sys)));
        println!("Elapsed: {:.2?}", now.elapsed());

        now = Instant::now();
        // collect_processes_info(&sys);
        println!("{}", serde_json::json!(collect_processes_info(&sys)));
        println!("Elapsed: {:.2?}", now.elapsed());

        now = Instant::now();
        println!("{}", serde_json::json!(get_docker_containers().await));
        println!("Elapsed: {:.2?}", now.elapsed());
    }
}
