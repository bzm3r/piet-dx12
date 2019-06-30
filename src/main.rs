// #![windows_subsystem = "windows"] (I think I want the console)
pub mod dx12;
pub mod error;
pub mod gpu;
pub mod window;

fn main() {
    unsafe {
        println!("creating window...");
        let mut wnd =
            window::Window::new(window::win32_string("test"), window::win32_string("test"));

        println!("beginning loop...");
        loop {
            if window::quit(&mut wnd) {
                println!("quitting...");
                break;
            }
            println!("    looping...");
        }
    }
}