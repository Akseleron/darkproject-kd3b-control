#[cfg(target_os = "linux")]
fn apply_linux_webkit_workarounds() {
    use std::{
        env,
        os::unix::process::CommandExt,
        path::Path,
        process::Command,
    };

    const WEBKIT_DMABUF_FLAG: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

    if env::var_os(WEBKIT_DMABUF_FLAG).is_some() {
        return;
    }

    let proprietary_nvidia_loaded = Path::new("/proc/driver/nvidia/version").exists()
        || Path::new("/sys/module/nvidia").exists();
    if !proprietary_nvidia_loaded {
        return;
    }

    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!(
                "KD3B Control: NVIDIA detected, but failed to resolve current executable for the WebKitGTK workaround: {error}"
            );
            return;
        }
    };

    eprintln!(
        "KD3B Control: NVIDIA driver detected; restarting with {WEBKIT_DMABUF_FLAG}=1 to avoid the known WebKitGTK DMABUF crash path."
    );

    let error = Command::new(current_exe)
        .args(env::args_os().skip(1))
        .env(WEBKIT_DMABUF_FLAG, "1")
        .exec();

    eprintln!("KD3B Control: failed to restart with the WebKitGTK workaround: {error}");
}

#[cfg(not(target_os = "linux"))]
fn apply_linux_webkit_workarounds() {}

fn main() {
    apply_linux_webkit_workarounds();
    kd3b_control_lib::run();
}
