extern crate crow;

fn main() {
    let mut global_context = crow::RenderingContext::new().unwrap();
    global_context.game_loop().unwrap();
}
