use crate::*;
use indexmap::IndexMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct Button {
    children: IndexMap<String, Widget>,
}
impl Button {
    pub fn new(text: String) -> Button {
        let id = Uuid::new_v4().to_string();
        let mut children = IndexMap::new();
        children.insert(
            id.clone(),
            TextField::new(text).wrap(id).placement(Placement::float()),
        );
        Button { children }
    }
}
impl Interactive for Button {
    fn handle_event(&mut self, _: WidgetEvent) -> bool {
        false
    }
    fn captures(&self) -> Capture {
        Capture {
            mouse: true,
            keyboard: false,
        }
    }
    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Widget> + 'a> {
        Box::new(self.children.values_mut())
    }
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &Widget> + 'a> {
        Box::new(self.children.values())
    }
    fn get_child(&self, id: &str) -> Option<&Widget> {
        self.children.get(id)
    }
    fn get_child_mut(&mut self, id: &str) -> Option<&mut Widget> {
        self.children.get_mut(id)
    }
    fn insert_child(&mut self, w: Widget) -> Option<()> {
        self.children.insert(w.get_id().to_string(), w);
        Some(())
    }
}

#[derive(Debug)]
pub struct ToggleButton {
    pub state: bool,
    children: IndexMap<String, Widget>,
}
impl ToggleButton {
    pub fn new(text: String) -> ToggleButton {
        let id = Uuid::new_v4().to_string();
        let mut children = IndexMap::new();
        children.insert(
            id.clone(),
            TextField::new(text).wrap(id).placement(Placement::float()),
        );
        ToggleButton {
            children,
            state: false,
        }
    }
}
impl Interactive for ToggleButton {
    fn handle_event(&mut self, event: WidgetEvent) -> bool {
        if let WidgetEvent::Release = event {
            self.state = !self.state;
            true
        } else {
            false
        }
    }
    fn captures(&self) -> Capture {
        Capture {
            mouse: true,
            keyboard: false,
        }
    }
    fn children_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &mut Widget> + 'a> {
        Box::new(self.children.values_mut())
    }
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = &Widget> + 'a> {
        Box::new(self.children.values())
    }
    fn get_child(&self, id: &str) -> Option<&Widget> {
        self.children.get(id)
    }
    fn get_child_mut(&mut self, id: &str) -> Option<&mut Widget> {
        self.children.get_mut(id)
    }
    fn insert_child(&mut self, w: Widget) -> Option<()> {
        self.children.insert(w.get_id().to_string(), w);
        Some(())
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;
    use crate::*;
    #[test]
    fn test_toggle_button_state() {
        let mut gui = single_toggle_button();

        // Frame 1: press
        let mut input = Input::default();
        press_left_mouse(&mut input);
        let (_events, _capture) = gui.update(&input, &mut ());

        // Frame 2: release
        input.prepare_for_next_frame();
        release_left_mouse(&mut input);
        let (events, _capture) = gui.update(&input, &mut ());
        assert_events!(events, vec![WidgetEvent::Release]);

        let btn = gui.get_widget("B1").unwrap();
        let btn = btn.downcast_ref::<ToggleButton>().unwrap();
        assert_eq!(btn.state, true);
    }
}
