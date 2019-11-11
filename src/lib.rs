#[macro_use]
extern crate mopa;
#[macro_use]
extern crate derive_deref;
use mopa::Any;
use winput::Input;

mod gui;
mod placement;
mod widgets;

pub use crate::gui::*;
pub use placement::*;
pub use widgets::*;

#[cfg(test)]
mod test;

#[derive(Deref, DerefMut, Debug)]
pub struct Widget {
    #[deref_target]
    pub inner: Box<dyn Interactive>,
    pub pos: (f32, f32),
    pub size: (f32, f32),

    /// Declarative placement (used to calculate position)
    pub place: Placement,
    pub anchor: (Anchor, Anchor),
    pub size_hint: SizeHint,

    /// Keeps track of hover state in order to generate the right WidgetEvents
    inside: bool,
    /// Keeps track of mouse press state in order to generate the right WidgetEvents
    pressed: bool,

    /// 'Buffer' - when `true` it is set to `false` by the parent, and the
    changed: bool,

    /// For internal use; mirrors the id that is the key in the HashMap that this Widget is
    /// likely a part of.
    /// NOTE: It's important to always ensure that `self.id` corresponds to the ID as registered in
    /// the gui system.
    id: String,
}

impl Widget {
    pub fn new<W: Interactive>(id: String, widget: W) -> Widget {
        let size_hint = widget.default_size_hint();
        Widget {
            inner: Box::new(widget),
            pos: (0.0, 0.0),
            size: (10.0, 10.0), // TODO Interactive::default_size()?
            place: Placement::Float(Axis::X, Anchor::Min),
            anchor: (Anchor::Min, Anchor::Min),
            size_hint,
            inside: false,
            pressed: false,
            changed: false,
            id,
        }
    }
    pub fn placement(mut self, place: Placement) -> Self {
        self.place = place;
        self
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    /// Mark that some internal state has changed in this Widget.
    /// For use when an application itself wants to change state of a Widget - for example toggle a
    /// button in response to a key press. A `Change` event has to be registered so that the drawer
    /// knows to redraw the widget.
    pub fn mark_change(&mut self) {
        self.changed = true;
    }
    /// Update this widget tree recursively, returning accumulated events from all nodes
    pub fn update(
        &mut self,
        input: &Input,
        sw: f32,
        sh: f32,
        mouse: (f32, f32),
    ) -> (Vec<(String, WidgetEventState)>, Capture) {
        macro_rules! event {
            ($event:expr, ($widget:expr, $id:expr, $events:expr)) => {{
                let change = $widget.inner.handle_event($event);
                if change {
                    $events.push((
                        $id.clone(),
                        WidgetEventState {
                            pressed: $widget.pressed,
                            hover: $widget.inside,
                            event: WidgetEvent::Change,
                        },
                    ));
                }
                $events.push((
                    $id.clone(),
                    WidgetEventState {
                        pressed: $widget.pressed,
                        hover: $widget.inside,
                        event: $event,
                    },
                ));
            }};
        }
        // Update positions of children (and possibly size of self)
        self.update_positions((sw, sh));

        // Update children
        let mut events = Vec::new();
        let mut capture = Capture::default();
        for child in self.children_mut() {
            let (child_events, child_capture) = child.update(input, sw, sh, mouse);
            capture |= child_capture;
            events.extend(child_events.into_iter());
        }

        if !capture.mouse {
            let now_inside = self.inside(self.pos, self.size, mouse);
            let prev_inside = self.inside;
            self.inside = now_inside;

            if now_inside && !prev_inside {
                event!(WidgetEvent::Hover, (self, self.id, events));
            } else if prev_inside && !now_inside {
                event!(WidgetEvent::Unhover, (self, self.id, events));
            }

            if now_inside {
                capture |= self.inner.captures();
            }

            if now_inside && input.is_mouse_button_toggled_down(winit::event::MouseButton::Left) {
                self.pressed = true;
                event!(WidgetEvent::Press, (self, self.id, events));
            }
            if self.pressed && input.is_mouse_button_toggled_up(winit::event::MouseButton::Left) {
                self.pressed = false;
                event!(WidgetEvent::Release, (self, self.id, events));
            }
        }

        if self.changed {
            events.push((
                self.id.clone(),
                WidgetEventState {
                    pressed: self.pressed,
                    hover: self.inside,
                    event: WidgetEvent::Change,
                },
            ));
            self.changed = false;
        }

        (events, capture)
    }

    /// Not recursive - only updates the position of children.
    /// (and updates size of `self` if applicable)
    fn update_positions(&mut self, screen: (f32, f32)) {
        let (pos, size) = (self.pos, self.size);
        let children = self.children_mut();
        let mut float_progress = pos.0;
        let mut max_width = 0.0;
        let mut max_height = 0.0;

        for widget in children {
            let pos = match widget.place {
                Placement::Fixed(Position {
                    x,
                    y,
                    x_anchor,
                    y_anchor,
                }) => (
                    match x_anchor {
                        Anchor::Min => pos.0 + x,
                        Anchor::Max => pos.0 + size.0 - x,
                        Anchor::Center => unimplemented!(),
                    },
                    match y_anchor {
                        Anchor::Min => pos.1 + y,
                        Anchor::Max => pos.1 + size.1 - y,
                        Anchor::Center => unimplemented!(),
                    },
                ),
                Placement::Float(axis, anchor) => {
                    if let (Axis::X, Anchor::Min) = (axis, anchor) {
                        float_progress += widget.size.0;
                        (float_progress - widget.size.0 / 2.0, 0.0)
                    } else {
                        unimplemented!();
                    }
                }
                Placement::Percentage(_x, _y) => unimplemented!(),
            };
            widget.pos = pos;
            if widget.pos.0 + widget.size.0 - pos.0 > max_width {
                max_width = widget.pos.0 + widget.size.0 - pos.0;
            }
            if widget.pos.1 + widget.size.1 - pos.1 > max_height {
                max_height = widget.pos.1 + widget.size.1 - pos.1;
            }
        }
        if let SizeHint::Minimize {
            top,
            bot,
            left,
            right,
        } = self.size_hint
        {
            self.size = (max_width + left + right, max_height + top + bot);
        }
    }
}

// TODO move to its own module. Problem with MOPA
/// An interactive component/node in the tree of widgets that defines a GUI. This is the trait that
/// all different widgets, such as buttons, checkboxes, containers, `Gui` itself, healthbars, ...,
/// implement.
pub trait Interactive: Any + std::fmt::Debug + Send + Sync {
    /// Defines an area which is considered "inside" a widget - for checking mouse hover etc.
    /// Provided implementation simply checks whether mouse is inside the boundaries, where `pos`
    /// is the very center of the widget. However, this is configurable in case a finer shape is
    /// desired (e.g. round things).
    fn inside(&self, pos: (f32, f32), size: (f32, f32), mouse: (f32, f32)) -> bool {
        let (x, y, w, h) = (pos.0, pos.1, size.0, size.1);
        let (top, bot, right, left) = (y + h / 2.0, y - h / 2.0, x + w / 2.0, x - w / 2.0);
        mouse.1 > bot && mouse.1 < top && mouse.0 > left && mouse.0 < right
    }
    /// Returns true if some internal state has changed in this widget (not in children)
    fn handle_event(&mut self, event: WidgetEvent) -> bool;

    /// Returns information whether this widget will stop mouse events and state
    /// to reach other parts of the application.
    fn captures(&self) -> Capture;

    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &Widget> + 'a>;
    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Widget> + 'a>;
    fn get_child(&mut self, id: &str) -> Option<&mut Widget>;
    fn insert_child(&mut self, id: String, w: Widget) -> Option<()>;

    /// Default size hint for this widget type. Defaults to `SizeHint::None`
    fn default_size_hint(&self) -> SizeHint {
        SizeHint::None
    }

    fn recursive_children_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Widget> + 'a> {
        Box::new(
            self.children().chain(
                self.children()
                    .map(|child| child.recursive_children_iter())
                    .flatten(),
            ),
        )
    }
}
mopafy!(Interactive);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WidgetEvent {
    Press,
    Release,
    Hover,
    Unhover,
    /// Change to any internal state
    Change,
    // TODO: perhaps something to notify that position has changed
}

#[derive(Clone, Debug)]
pub struct WidgetEventState {
    pub hover: bool,
    pub pressed: bool,
    pub event: WidgetEvent,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Capture {
    pub mouse: bool,
    pub keyboard: bool,
}
impl std::ops::BitOrAssign for Capture {
    fn bitor_assign(&mut self, rhs: Self) {
        self.mouse |= rhs.mouse;
        self.keyboard |= rhs.keyboard;
    }
}
