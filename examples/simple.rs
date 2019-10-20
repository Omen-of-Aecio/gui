use gui::{Pos, FromWidget};
use gui::{Gui, Button};
#[macro_use]
extern crate derive_deref;


use vxdraw::{void_logger, ShowWindow, VxDraw, Color};

// use cgmath::SquareMatrix;
// use cgmath::Matrix4;
use winput::Input;
use gui_drawer::GuiDrawer;

#[macro_export]
macro_rules! match_downcast_ref {
    ( $any:expr, $( $bind:ident : $ty:ty => $arm:expr ),*, _ => $default:expr ) => (
        $(
            if $any.is::<$ty>() {
                let $bind = $any.downcast_ref::<$ty>().unwrap();
                $arm
            } else
        )*
        {
            $default
        }
    )
}

fn main() {
    let mut vx = VxDraw::new(void_logger(), ShowWindow::Enable);
    vx.set_clear_color(Color::Rgba(0,0,0, 255));
    
    // let world_layer = vx.quads().add_layer(&vxdraw::quads::LayerOptions::new()
        // .fixed_perspective(Matrix4::identity()));

    // Create GUI
    let mut gui = Gui::default();

    gui.insert(Button1, Button::new("B1".to_string(), 60, 30),
        Pos(100), Pos(100));
    gui.insert(Button2, Button::new("B2".to_string(), 60, 30),
        FromWidget (Button1, 0),
        FromWidget (Button1, 100));

    let mut gui = GuiDrawer::new(gui, &mut vx);
    // TODO: make it possible to add widgets after creating GuiDrawer


    loop {
        let prspect = vx.perspective_projection();
        vx.set_perspective(prspect);

        gui.update(&Input::default() /*TODO*/, &mut vx);

        vx.draw_frame();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

use WidgetId::*;
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum WidgetId {
    Button1,
    Button2,
}
impl Default for WidgetId {
    fn default() -> Self {
        WidgetId::Button1
    }
}

mod gui_drawer {
    use vxdraw::quads::Handle;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::ops::{Deref, DerefMut};
    use std::fmt::Debug;
    use gui::{self, Widget, Gui};
    use cgmath::SquareMatrix;

    use vxdraw::{
        debtri::DebugTriangle,
        quads,
        text,
        void_logger,
        ShowWindow,
        VxDraw,
        Color
    };
    use cgmath::{Matrix4, Vector3, Transform};
    use winput::Input;

    const DEJAVU: &[u8] = include_bytes!["../fonts/DejaVuSans.ttf"];

    fn vx_transform(pos: (i32, i32), sw: i32, sh: i32) -> (f32, f32) {
        (pos.0 as f32 / sw as f32 * 2.0 - 1.0,
         pos.1 as f32 / sh as f32 * 2.0 - 1.0)
    }
    fn vx_scale(x: i32, sw: i32) -> f32 {
        x as f32 / sw as f32 * 2.0
    }

    struct Button {
        quad: quads::Handle,
        text: text::Handle,
    }

    #[derive(Deref, DerefMut)]
    pub struct GuiDrawer<Id: Eq + Hash> {
        #[deref_target]
        gui: Gui<Id>,

        // Handles to VxDraw
        quads: quads::Layer,
        text: text::Layer,
        buttons: HashMap<Id, Button>,
    }
    impl<Id: Eq + Hash + Copy + Clone + Debug> GuiDrawer<Id> {
        fn proj_matrix(vx: &mut VxDraw) -> Matrix4<f32> {
            let (sw, sh) = vx.get_window_size_in_pixels();
            let (sw, sh) = (sw as f32, sh as f32);
            // transform (0,0) -> (-1,-1)
            // transform (sw,sh) -> (1,1)
            Matrix4::from_translation(Vector3::new(-1.0, -1.0, 0.0))
                * Matrix4::from_nonuniform_scale(2.0/sw, 2.0/sh, 1.0)
        }

        pub fn new(mut gui: Gui<Id>, vx: &mut VxDraw) -> GuiDrawer<Id> {
            let proj_matrix = Self::proj_matrix(vx);
            let quads = vx.quads().add_layer(&vxdraw::quads::LayerOptions::new()
                .fixed_perspective(proj_matrix));
            let mut text = vx.text().add_layer(DEJAVU, text::LayerOptions::new()
                .fixed_perspective(proj_matrix));

            let mut buttons = HashMap::new();

            let (sw, sh) = vx.get_window_size_in_pixels();
            let (sw, sh) = (sw as i32, sh as i32);

            gui.update(&Input::default(), sw, sh);

            // Initiate state
            for (id, w) in gui.widgets.iter() {
                let pos = w.pos;
                let widget = &w.widget;
                match_downcast_ref! {widget,
                    button: gui::Button => {
                        println!("Adding button");
                        println!("Pos: {:?}", widget.deref().deref());
                        let quad = vx.quads().add(&quads, vxdraw::quads::Quad::new()
                            .translation(pos.to_tuple())
                            .width(button.w as f32)
                            .height(button.h as f32));
                        vx.quads().set_solid_color(&quad, Color::Rgba(128,128,128, 255));

                        let text = vx.text().add(
                            &text,
                            &button.text,
                            text::TextOptions::new()
                                .font_size(30.0)
                                .translation(pos.to_tuple())
                        );
                        buttons.insert(*id, Button {
                            quad,
                            text,
                        });
                    },
                    _ => panic!("Unexpected Widget!")
                }
            }

            GuiDrawer {
                gui,
                quads,
                text,
                buttons,
            }
        }
        pub fn update(&mut self, input: &Input, vx: &mut VxDraw) {
            let (sw, sh) = vx.get_window_size_in_pixels();
            let (sw, sh) = (sw as i32, sh as i32);

            self.gui.update(input, sw, sh);

            let proj_matrix = Self::proj_matrix(vx);
            vx.quads().set_perspective(&self.quads, Some(proj_matrix));
            vx.text().set_perspective(&self.text, Some(proj_matrix));

            for (id, w) in self.gui.widgets.iter() {
                let pos = w.pos;
                let widget = &w.widget;
                match_downcast_ref! {widget,
                    button: gui::Button => {
                        // TODO
                    },
                    _ => panic!("Unexpected Widget!")
                }
            }
        }
    }
}
