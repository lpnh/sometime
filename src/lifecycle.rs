#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    // No surface, the daemon idles waiting for commands
    Sleep,
    // A layer surface was requested, waiting for the compositor to
    /// configure it before drawing
    WakeUp(View),
    // The view is on screen, redrawn as time passes
    Awake(View),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Clock,
    Calendar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    // The `clock` or `calendar` command
    Toggle(View),
    // The compositor acknowledged the layer surface
    Configure,
    // `esc`/`q`, a key release under `--exit-on-release`, or the compositor closing the surface
    Close,
    // The `dismiss` command
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    CreateLayer,
    Draw,
    DestroyLayer,
    Vanish,
    Ignore,
}

impl State {
    pub fn and_then(self, event: Event) -> (Self, Action) {
        match (self, event) {
            // The lifecycle: Sleep -> WakeUp -> Awake -> Sleep
            (Self::Sleep, Event::Toggle(view)) => (Self::WakeUp(view), Action::CreateLayer),
            (Self::WakeUp(view), Event::Configure) => (Self::Awake(view), Action::Draw),
            (Self::Awake(current_view), Event::Toggle(view)) if view == current_view => {
                (Self::Sleep, Action::DestroyLayer)
            }

            // Close and quit regardless of current state
            (_, Event::Close) => (Self::Sleep, Action::DestroyLayer),
            (state, Event::Quit) => (state, Action::Vanish),

            // Anything else is ignored
            (state, _) => (state, Action::Ignore),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action::*, Event::*, State::*, View::*};

    #[test]
    fn lifecycle() {
        // Clock
        assert_eq!(Sleep.and_then(Toggle(Clock)), (WakeUp(Clock), CreateLayer));
        assert_eq!(WakeUp(Clock).and_then(Configure), (Awake(Clock), Draw));
        assert_eq!(Awake(Clock).and_then(Toggle(Clock)), (Sleep, DestroyLayer));

        // Calendar
        assert_eq!(
            Sleep.and_then(Toggle(Calendar)),
            (WakeUp(Calendar), CreateLayer)
        );
        assert_eq!(
            WakeUp(Calendar).and_then(Configure),
            (Awake(Calendar), Draw)
        );
        assert_eq!(
            Awake(Calendar).and_then(Toggle(Calendar)),
            (Sleep, DestroyLayer)
        );
    }

    #[test]
    fn ignore_configure_when_sleep_or_awake() {
        // Sleep
        assert_eq!(Sleep.and_then(Configure), (Sleep, Ignore));

        // Clock
        assert_eq!(Awake(Clock).and_then(Configure), (Awake(Clock), Ignore));

        // Calendar
        assert_eq!(
            Awake(Calendar).and_then(Configure),
            (Awake(Calendar), Ignore)
        );
    }

    #[test]
    fn ignore_toggle_when_wake_up() {
        // Clock
        assert_eq!(
            WakeUp(Clock).and_then(Toggle(Clock)),
            (WakeUp(Clock), Ignore)
        );
        assert_eq!(
            WakeUp(Clock).and_then(Toggle(Calendar)),
            (WakeUp(Clock), Ignore)
        );

        // Calendar
        assert_eq!(
            WakeUp(Calendar).and_then(Toggle(Calendar)),
            (WakeUp(Calendar), Ignore)
        );
        assert_eq!(
            WakeUp(Calendar).and_then(Toggle(Clock)),
            (WakeUp(Calendar), Ignore)
        );
    }

    #[test]
    fn ignore_toggle_for_different_view_when_awake() {
        // Clock
        assert_eq!(
            Awake(Clock).and_then(Toggle(Calendar)),
            (Awake(Clock), Ignore)
        );

        // Calendar
        assert_eq!(
            Awake(Calendar).and_then(Toggle(Clock)),
            (Awake(Calendar), Ignore)
        );
    }

    #[test]
    fn always_destroy_layer_and_sleep_after_close() {
        // Sleep
        assert_eq!(Sleep.and_then(Close), (Sleep, DestroyLayer));

        // Clock
        assert_eq!(WakeUp(Clock).and_then(Close), (Sleep, DestroyLayer));
        assert_eq!(Awake(Clock).and_then(Close), (Sleep, DestroyLayer));

        // Calendar
        assert_eq!(WakeUp(Calendar).and_then(Close), (Sleep, DestroyLayer));
        assert_eq!(Awake(Calendar).and_then(Close), (Sleep, DestroyLayer));
    }

    #[test]
    fn always_vanish_after_quit() {
        // Sleep
        assert_eq!(Sleep.and_then(Quit), (Sleep, Vanish));

        // Clock
        assert_eq!(WakeUp(Clock).and_then(Quit), (WakeUp(Clock), Vanish));
        assert_eq!(Awake(Clock).and_then(Quit), (Awake(Clock), Vanish));

        // Calendar
        assert_eq!(WakeUp(Calendar).and_then(Quit), (WakeUp(Calendar), Vanish));
        assert_eq!(Awake(Calendar).and_then(Quit), (Awake(Calendar), Vanish));
    }
}
