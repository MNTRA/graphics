use application::{Application, ApplicationContext, run_app};
use windowing::{
    window::{WindowContext, WindowId, Window},
    WindowingPlugin,
};

#[derive(Default, Debug)]
pub struct MainWindow;
impl Window for MainWindow {
    fn on_create(&self, mut ctx: WindowContext<'_>) {
        ctx.set_title_text("My Super Cool Window");
    }
    fn close_requested(&mut self, _: WindowContext<'_>) -> bool {
        true
    }
    fn on_destroyed(&mut self, mut ctx: WindowContext<'_>) {
        ctx.post_shutdown_message();
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
        // ctx.add_plugin::<RenderingPlugin>();

        ctx.window = Some(ctx.create_window(MainWindow::default()));
    }
}

run_app!(BasicApplication);