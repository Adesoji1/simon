export const types2names: { [key: string]: string } = {
    cpu_usage: 'CPU Usage',
    mem_usage: 'Memory Usage',
    swap_usage: 'Swap Usage',
    load_avg_1: '1 min Load Average',
    load_avg_5: '5 min Load Average',
    load_avg_15: '15 min Load Average',
    rx_rate: 'RX Rate',
    tx_rate: 'TX Rate',
    read_rate: 'Read Rate',
    write_rate: 'Write Rate',
    total_read: 'Total Read',
    total_write: 'Total Write',
    disk_usage: 'Disk Usage',
    rx: 'RX',
    tx: 'TX'
};

export const cat2names: { [key: string]: string } = {
    sys: "System",
    net: "Network",
    disk: "Storage",
};

export const var2unit: { [key: string]: string } = {
    cpu_usage: '%',
    mem_usage: '%',
    swap_usage: '%',
    load_avg_1: '',
    load_avg_5: '',
    load_avg_15: '',
    rx_rate: 'B/s',
    tx_rate: 'B/s',
    read_rate: 'B/s',
    write_rate: 'B/s',
    total_read: 'B',
    total_write: 'B',
    disk_usage: '%',
    rx: 'B',
    tx: 'B'
};

export function ws_url(s: string) {
    var l = window.location;
    return ((l.protocol === "https:") ? "wss://" : "ws://") + l.host + "/" + s;
}

export function url(s: string) {
    var l = window.location;
    return l.origin + "/" + s;
}

// Format bytes to human readable format
export function formatBytes(bytes: number, decimals: number = 1) {
    bytes = Math.floor(bytes);
    if (bytes <= 0) return '0 Bytes';
    
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];
    
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

export function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    seconds %= 86400;
    const hours = Math.floor(seconds / 3600);
    seconds %= 3600;
    const minutes = Math.floor(seconds / 60);
    seconds %= 60;
    if (days > 0) {
        return `${days}d ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

// Format bytes per second with appropriate unit
export function formatBytesPerSecond(bytesPerSecond: number, decimals: number = 1) {
    bytesPerSecond = Math.floor(bytesPerSecond);
    if (bytesPerSecond <= 0) return '0 B/s';
    
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s'];
    
    const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
    
    return parseFloat((bytesPerSecond / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}