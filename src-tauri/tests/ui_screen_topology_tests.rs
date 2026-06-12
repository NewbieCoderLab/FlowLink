use flowlink_lib::{
    input::types::{DisplayInfo, Rect, ScreenTopology},
    ui_api::models::UiScreenTopology,
};

#[test]
fn ui_screen_topology_preserves_display_geometry() {
    let topology = ScreenTopology {
        displays: vec![DisplayInfo {
            id: 7,
            bounds: Rect {
                x: -1920.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
            scale_factor: 1.25,
            is_primary: false,
        }],
        virtual_bounds: Rect {
            x: -1920.0,
            y: 0.0,
            width: 3840.0,
            height: 1080.0,
        },
    };

    let ui = UiScreenTopology::from(&topology);

    assert_eq!(ui.displays.len(), 1);
    assert_eq!(ui.displays[0].id, 7);
    assert_eq!(ui.displays[0].bounds.x, -1920.0);
    assert_eq!(ui.displays[0].scale_factor, 1.25);
    assert!(!ui.displays[0].is_primary);
    assert_eq!(ui.virtual_bounds.width, 3840.0);
}
