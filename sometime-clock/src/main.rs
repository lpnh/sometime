mod canvas;
mod clock;
use clock::Clock;

use sometime::{impl_registry_handlers, run_app};

impl_registry_handlers!(Clock);

fn main() {
    run_app::<Clock>();
}
