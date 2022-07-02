use windowing::{
    window::{Window, WindowContext, WindowId},
    Application, ApplicationContext,
};

#[derive(Default, Debug)]
pub struct MainWindow {
    count: usize,
}
impl Window for MainWindow {
    fn on_create(&self, mut ctx: WindowContext<'_>) {
        ctx.set_window_title("Main Window");
    }
    fn close_requested(&mut self, _: WindowContext<'_>) -> bool {
        true
    }
}

#[derive(Default)]
struct BasicApplication {
    window: Option<WindowId>,
}

impl Application for BasicApplication {
    fn initalize(&mut self, ctx: &mut ApplicationContext) {
        self.window = Some(ctx.create_window(MainWindow::default()));
    }
}

fn main() -> ! {
    windowing::run_application(BasicApplication::default());
}
