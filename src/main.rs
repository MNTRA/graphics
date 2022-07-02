use application::{Application, ApplicationContext, run_app};
use renderer::RenderingPlugin;
use windowing::{
    window::{WindowContext, WindowId, Window},
    WindowingPlugin,
};

#[derive(Default, Debug)]
pub struct MainWindow;
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
    type Data = Self;
    fn initalize(mut ctx: ApplicationContext<'_, Self::Data>) {
        ctx.add_plugin::<WindowingPlugin>();
        ctx.add_plugin::<RenderingPlugin>();

        ctx.window = Some(ctx.create_window(MainWindow::default()));
    }
}

run_app!(BasicApplication);