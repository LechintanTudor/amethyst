use crate::{
<<<<<<< HEAD
    Anchor, FontAsset, Parent, Stretch, UiButton, UiButtonAction, UiButtonActionRetrigger,
    UiButtonActionType, UiImage, UiText, UiTransform,
=======
    font::default::get_default_font,
    Anchor, FontAsset, FontHandle, Interactable, LineMode, Selectable, Stretch, UiButton,
    UiButtonAction, UiButtonActionRetrigger,
    UiButtonActionType::{self, *},
    UiImage, UiPlaySoundAction, UiSoundRetrigger, UiText, UiTransform, WidgetId, Widgets,
>>>>>>> origin/legion_v2
};
use amethyst_assets::Handle;
use amethyst_core::ecs::{prelude::*, storage::Component};
use amethyst_rendy::palette::Srgba;
use smallvec::SmallVec;

const DEFAULT_Z: f32 = 1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;
const DEFAULT_FONT_SIZE: f32 = 32.0;
const DEFAULT_TEXT_COLOR: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 1.0);
const DEFAULT_BACKGROUND_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

/// Trait implemented by types which support creating entities
/// and adding components to them
pub trait UiButtonBuilderTarget {
    /// Creates a new entity.
    fn create_entity(&mut self) -> Entity;

    /// Adds a component to an existing entity.
    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component;
}

impl UiButtonBuilderTarget for World {
    fn create_entity(&mut self) -> Entity {
        self.insert((), Some(()))[0]
    }

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        World::add_component(self, entity, component).unwrap();
    }
}

<<<<<<< HEAD
impl UiButtonBuilderTarget for CommandBuffer {
    fn create_entity(&mut self) -> Entity {
        self.insert((), Some(()))[0]
    }

    fn add_component<C>(&mut self, entity: Entity, component: C)
    where
        C: Component,
    {
        CommandBuffer::add_component(self, entity, component);
    }
}

/// Convenience structure for building a `UiButton`
#[derive(Clone, Debug)]
pub struct UiButtonBuilder {
=======
/// Convenience structure for building a button
/// Note that since there can only be one "ui_loader" in use, and WidgetId of the UiBundle and
/// UiButtonBuilder should match, you can only use one type of WidgetId, e.g. you cant use both
/// UiButtonBuilder<(), u32> and UiButtonBuilder<(), String>.
#[derive(Debug, Clone)]
pub struct UiButtonBuilder<G, I: WidgetId> {
    id: Option<I>,
>>>>>>> origin/legion_v2
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    anchor: Anchor,
    pivot: Anchor,
    stretch: Stretch,
    text: String,
    text_color: Srgba,
    font: Option<Handle<FontAsset>>,
    font_size: f32,
<<<<<<< HEAD
    image: UiImage,
=======
    line_mode: LineMode,
    align: Anchor,
    image: Option<UiImage>,
>>>>>>> origin/legion_v2
    parent: Option<Entity>,
    on_click_start: SmallVec<[UiButtonActionType; 2]>,
    on_click_stop: SmallVec<[UiButtonActionType; 2]>,
    on_hover_start: SmallVec<[UiButtonActionType; 2]>,
    on_hover_stop: SmallVec<[UiButtonActionType; 2]>,
}

impl Default for UiButtonBuilder {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            anchor: Anchor::Middle,
            pivot: Anchor::Middle,
            stretch: Stretch::NoStretch,
            text: String::new(),
            text_color: Srgba::from_components(DEFAULT_TEXT_COLOR),
            font: None,
<<<<<<< HEAD
            font_size: DEFAULT_FONT_SIZE,
            image: UiImage::SolidColor(Srgba::from_components(DEFAULT_BACKGROUND_COLOR)),
=======
            font_size: 32.,
            line_mode: LineMode::Single,
            align: Anchor::Middle,
            image: None,
>>>>>>> origin/legion_v2
            parent: None,
            on_click_start: SmallVec::new(),
            on_click_stop: SmallVec::new(),
            on_hover_start: SmallVec::new(),
            on_hover_stop: SmallVec::new(),
        }
    }
}

impl UiButtonBuilder {
    /// Sets the position of the button.
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Sets the layer of the button. Widgets with higher Z values
    /// get rendered above widgets with lower Z values.
    pub fn with_layer(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Sets the size of the button.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the anchor of the button. The anchor is the origin
    /// of the coordinate system on the parent of the button
    /// or the screen if no parent was provided.
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the pivot of the button. The pivot is the point on
    /// the widget that is pinned to the provided position.
    pub fn with_pivot(mut self, pivot: Anchor) -> Self {
        self.pivot = pivot;
        self
    }

    /// Sets the way the button stretches relative to its parent
    /// or screen if no parent was provided
    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    /// Sets the text of the button.
    pub fn with_text<S>(mut self, text: S) -> Self
    where
        S: ToString,
    {
        self.text = text.to_string();
        self
    }

    /// Sets the text color of the button.
    pub fn with_text_color(mut self, text_color: Srgba) -> Self {
        self.text_color = text_color;
        self
    }

    /// Sets the text color of the button when the button is hovered.
    pub fn with_hover_text_color(mut self, text_color: Srgba) -> Self {
        self.on_hover_start
            .push(UiButtonActionType::SetTextColor(text_color));
        self.on_hover_stop
            .push(UiButtonActionType::UnsetTextColor(text_color));
        self
    }

    /// Sets the text color of the button when the button is pressed.
    pub fn with_press_text_color(mut self, text_color: Srgba) -> Self {
        self.on_click_start
            .push(UiButtonActionType::SetTextColor(text_color));
        self.on_click_stop
            .push(UiButtonActionType::UnsetTextColor(text_color));
        self
    }

    /// Sets the font of the button.
    pub fn with_font(mut self, font: Handle<FontAsset>) -> Self {
        self.font = Some(font);
        self
    }

<<<<<<< HEAD
    /// Sets the font size of the button.
    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
=======
    /// Set text line mode
    pub fn with_line_mode(mut self, line_mode: LineMode) -> Self {
        self.line_mode = line_mode;
        self
    }

    /// Set text align
    pub fn with_align(mut self, align: Anchor) -> Self {
        self.align = align;
        self
    }

    /// Text color to use when the mouse is hovering over this button
    pub fn with_hover_text_color(mut self, text_color: [f32; 4]) -> Self {
        self.on_hover_start.push(SetTextColor(text_color));
        self.on_hover_stop.push(UnsetTextColor(text_color));
>>>>>>> origin/legion_v2
        self
    }

    /// Sets a `UiImage` to be displayed as a background.
    pub fn with_image(mut self, image: UiImage) -> Self {
        self.image = image;
        self
    }

    /// Sets a `UiImage` to to be displayed as a background when the
    /// button is hovered.
    pub fn with_hover_image(mut self, image: UiImage) -> Self {
        self.on_hover_start
            .push(UiButtonActionType::SetImage(image.clone()));
        self.on_hover_stop
            .push(UiButtonActionType::UnsetImage(image));
        self
    }

    /// Sets a `UiImage` to to be displayed as a background when the
    /// button is pressed.
    pub fn with_press_image(mut self, image: UiImage) -> Self {
        self.on_click_start
            .push(UiButtonActionType::SetImage(image.clone()));
        self.on_click_stop
            .push(UiButtonActionType::UnsetImage(image));
        self
    }

    /// Sets the parent of the button.
    pub fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Builds the button.
    pub fn build<T>(self, target: &mut T) -> UiButton
    where
        T: UiButtonBuilderTarget,
    {
        let entity = target.create_entity();

        target.add_component(
            entity,
            UiTransform::new(
                "PLACEHOLDER",
                self.anchor,
                self.pivot,
                self.x,
                self.y,
                self.z,
                self.width,
                self.height,
            ),
        );

        if let Some(parent) = self.parent {
            target.add_component(entity, Parent(parent));
        }

        target.add_component(entity, self.image);

        target.add_component(
            entity,
            UiText::new(
                self.font.expect("TODO: Implement default font"),
                self.text,
                self.text_color,
                self.font_size,
            ),
        );

        if !self.on_click_start.is_empty()
            || !self.on_click_stop.is_empty()
            || !self.on_hover_start.is_empty()
            || !self.on_hover_stop.is_empty()
        {
<<<<<<< HEAD
            target.add_component(
                entity,
                UiButtonActionRetrigger {
                    on_click_start: actions_with_target(self.on_click_start.into_iter(), entity),
                    on_click_stop: actions_with_target(self.on_click_stop.into_iter(), entity),
                    on_hover_start: actions_with_target(self.on_hover_start.into_iter(), entity),
                    on_hover_stop: actions_with_target(self.on_hover_stop.into_iter(), entity),
=======
            let retrigger = UiButtonActionRetrigger {
                on_click_start: actions_with_target(
                    &mut self.on_click_start.into_iter(),
                    image_entity,
                ),
                on_click_stop: actions_with_target(
                    &mut self.on_click_stop.into_iter(),
                    image_entity,
                ),
                on_hover_start: actions_with_target(
                    &mut self.on_hover_start.into_iter(),
                    image_entity,
                ),
                on_hover_stop: actions_with_target(
                    &mut self.on_hover_stop.into_iter(),
                    image_entity,
                ),
            };

            res.button_action_retrigger
                .insert(image_entity, retrigger)
                .expect("Unreachable: Inserting newly created entity");
        }

        if self.on_click_start_sound.is_some()
            || self.on_click_stop_sound.is_some()
            || self.on_hover_sound.is_some()
        {
            let retrigger = UiSoundRetrigger {
                on_click_start: self.on_click_start_sound,
                on_click_stop: self.on_click_stop_sound,
                on_hover_start: self.on_hover_sound,
                on_hover_stop: None,
            };

            res.sound_retrigger
                .insert(image_entity, retrigger)
                .expect("Unreachable: Inserting newly created entity");
        }

        res.transform
            .insert(
                image_entity,
                UiTransform::new(
                    format!("{}_btn", id),
                    self.anchor,
                    Anchor::Middle,
                    self.x,
                    self.y,
                    self.z,
                    self.width,
                    self.height,
                )
                .with_stretch(self.stretch),
            )
            .expect("Unreachable: Inserting newly created entity");
        res.selectables
            .insert(image_entity, Selectable::<G>::new(self.tab_order))
            .expect("Unreachable: Inserting newly created entity");
        let image = self.image.unwrap_or_else(|| {
            UiImage::Texture(
                res.loader.load_from_data(
                    load_from_srgba(Srgba::new(
                        DEFAULT_BKGD_COLOR[0],
                        DEFAULT_BKGD_COLOR[1],
                        DEFAULT_BKGD_COLOR[2],
                        DEFAULT_BKGD_COLOR[3],
                    ))
                    .into(),
                    (),
                    &res.texture_asset,
                ),
            )
        });

        res.image
            .insert(image_entity, image)
            .expect("Unreachable: Inserting newly created entity");
        res.mouse_reactive
            .insert(image_entity, Interactable)
            .expect("Unreachable: Inserting newly created entity");
        if let Some(parent) = self.parent.take() {
            res.parent
                .insert(image_entity, Parent { entity: parent })
                .expect("Unreachable: Inserting newly created entity");
        }

        res.transform
            .insert(
                text_entity,
                UiTransform::new(
                    format!("{}_btn_text", id),
                    Anchor::Middle,
                    Anchor::Middle,
                    0.,
                    0.,
                    0.01,
                    0.,
                    0.,
                )
                .into_transparent()
                .with_stretch(Stretch::XY {
                    x_margin: 0.,
                    y_margin: 0.,
                    keep_aspect_ratio: false,
                }),
            )
            .expect("Unreachable: Inserting newly created entity");
        let font_handle = self
            .font
            .unwrap_or_else(|| get_default_font(&res.loader, &res.font_asset));
        res.text
            .insert(
                text_entity,
                UiText::new(
                    font_handle,
                    self.text,
                    self.text_color,
                    self.font_size,
                    self.line_mode,
                    self.align,
                ),
            )
            .expect("Unreachable: Inserting newly created entity");
        res.parent
            .insert(
                text_entity,
                Parent {
                    entity: image_entity,
>>>>>>> origin/legion_v2
                },
            );
        }

        UiButton::new(entity)
    }
}

fn actions_with_target<I>(actions: I, target: Entity) -> Vec<UiButtonAction>
where
    I: Iterator<Item = UiButtonActionType>,
{
    actions
        .map(|action| UiButtonAction::new(action, target))
        .collect()
}
