use log::trace;
use probset::render::Model;
use yew::App;

fn main() {
    web_logger::init();
    trace!("init yew...");
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
