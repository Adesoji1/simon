export enum wsStatus{
  INIT = 0,
  WAITING = 1,
  CONNECTED = 2,
  DISCONNECTED = 3,
  ERROR = 4,
}

export interface SystemData {
  sys: SystemInfo;
  mem: MemoryInfo;
  cpu: CPUInfo;
  net: NetworkInfo;
  disk: DiskInfo;
}

interface SystemInfo {
  name: string;
  kernel_ver: string;
  os_ver: string;
  os_name: string;
  host_name: string;
  load_avg: number[];
  uptime: number;
}

interface MemoryInfo {
  total_mem: number;
  used_mem: number;
  total_swap: number;
  used_swap: number;
}

interface CPUInfo {
  count: number;
  avg_usage: number;
  usage: number[];
}

interface NetworkInfo {
  interfaces: NetworkInterface[];
}

interface NetworkInterface {
  name: string;
  rx: number;
  tx: number;
}

interface DiskInfo {
  disks: Disk[];
}

export interface Disk {
  fs: string;
  kind: string;
  total_space: number;
  free_space: number;
  mount_point: string;
  removable: boolean;
  io: number[];
}


export interface DockerPort {
  ip?: string;
  priv_port: number;
  pub_port?: number;
  protocol: string;
}

export interface DockerContainer {
  id: string;
  name: string;
  image: string;
  status: string;
  state: string;
  created: number;
  ports: DockerPort[];
  cpu_usage: number;
  mem_usage: number;
  mem_limit: number;
  net_io: [number, number];
  disk_io: [number, number];
}

export interface DockerInfo {
  containers: DockerContainer[];
}

export interface NotificationMethod {
  id: string;
  name: string;
  kind: string;
  enabled: boolean;
  config: {
    WebHook: {
      url: string;
      method: string;
      headers: Record<string, string>;
      body: string;
    };
  };
}


export interface AlertVar {
   cat: string,   // Category
   var: string,   // variable name (ex. rx_rate)
   resrc: string, // Resource name (ex. eth0)
}

export interface Alert {
   id: string,
   var: AlertVar,
   threshold: number,
   operator: string,
   enabled: boolean,
   time_window: number,
   firing: boolean,
   notif_methods: string[],
}