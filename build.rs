extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/buddy_alloc.c")
        .compile("buddy_alloc.a");
}
