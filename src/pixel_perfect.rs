// This provides a plugin to achieve pixel-perfect rendering
// 1. First the 2d renderer renders all sprites to a texture with the
// desired pixel dimensions.
// 2. Then the 3d renderer renders a quad covering the screen with the texture
// This is just a hacky stopgap until bevy adds better support for this
// use case. Hopefully 0.7's render target improvements will let most
// of this code go away.


use bevy::{
    core_pipeline::{
        draw_2d_graph, node, RenderTargetClearColors, Transparent2d,
    },
    prelude::*,
    render::{
        camera::{ActiveCamera, CameraTypePlugin, RenderTarget, Camera3d, ScalingMode},
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotValue},
        render_phase::RenderPhase,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderContext,
        RenderApp, RenderStage,
    },
};

pub const PIXELS_PER_TILE: u32 = 16;

#[derive(Component, Default)]
pub struct WorldCamera;

// The name of the final node of the first pass.
pub const FIRST_PASS_DRIVER: &str = "first_pass_driver";

#[derive(Default)]
pub struct PixelPerfectPlugin;

impl Plugin for PixelPerfectPlugin {
    fn build(&self, app: &mut App) {
      app.insert_resource(Msaa { samples: 1 }) // Use 4x MSAA
          .add_plugin(CameraTypePlugin::<WorldCamera>::default())
          .add_startup_system(setup);

      let render_app = app.sub_app_mut(RenderApp);
      let driver = WorldCameraDriver::new(&mut render_app.world);
      render_app.add_system_to_stage(RenderStage::Extract, extract_first_pass_camera_phases);

      let mut graph = render_app.world.resource_mut::<RenderGraph>();

      // Add a node for the first pass.
      graph.add_node(FIRST_PASS_DRIVER, driver);

      // The first pass's dependencies include those of the main pass.
      graph
          .add_node_edge(node::MAIN_PASS_DEPENDENCIES, FIRST_PASS_DRIVER)
          .unwrap();

      // Insert the first pass node: CLEAR_PASS_DRIVER -> FIRST_PASS_DRIVER -> MAIN_PASS_DRIVER
      graph
          .add_node_edge(node::CLEAR_PASS_DRIVER, FIRST_PASS_DRIVER)
          .unwrap();
      graph
          .add_node_edge(FIRST_PASS_DRIVER, node::MAIN_PASS_DRIVER)
          .unwrap();
    }
}

fn extract_first_pass_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<WorldCamera>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Transparent2d>::default(),
        ));
    }
}

struct WorldCameraDriver {
    query: QueryState<Entity, With<WorldCamera>>,
}

impl WorldCameraDriver {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl Node for WorldCameraDriver {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for camera in self.query.iter_manual(world) {
            graph.run_sub_graph(draw_2d_graph::NAME, vec![SlotValue::Entity(camera)])?;
        }
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut clear_colors: ResMut<RenderTargetClearColors>,
) {
    let size = Extent3d {
        width: 16 * PIXELS_PER_TILE * 2,
        height: 9 * PIXELS_PER_TILE * 2,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    // First pass camera
    let render_target = RenderTarget::Image(image_handle.clone());
    clear_colors.insert(render_target.clone(), Color::BLACK);

    let mut cam_2d = OrthographicCameraBundle::new_2d();
    cam_2d.camera.target = render_target;
    cam_2d.transform.scale = Vec3::new(1. / PIXELS_PER_TILE as f32, 1. / PIXELS_PER_TILE as f32, 1.);
    commands
        .spawn_bundle(OrthographicCameraBundle::<WorldCamera> {
            camera: cam_2d.camera,
            orthographic_projection: cam_2d.orthographic_projection,
            visible_entities: cam_2d.visible_entities,
            frustum: cam_2d.frustum,
            transform: cam_2d.transform,
            global_transform: cam_2d.global_transform,
            marker: WorldCamera,
        });

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(2. * 16. / 9., 2.))));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        unlit: true,
        ..default()
    });
    commands
        .spawn_bundle(PbrBundle {
            mesh: quad_handle,
            material: material_handle,
            ..default()
        });

    // The main pass camera.
    let cam_2d = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(OrthographicCameraBundle {
        camera: cam_2d.camera,
        orthographic_projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical,
            ..cam_2d.orthographic_projection
        },
        visible_entities: cam_2d.visible_entities,
        frustum: cam_2d.frustum,
        transform: cam_2d.transform,
        global_transform: cam_2d.global_transform,
        marker: Camera3d,
    });
}
