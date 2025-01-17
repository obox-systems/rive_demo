//! A demonstration application that renders Rive animations using Vello renderer
//! and Winit window management.

use rive_rs::Instantiate;

fn main()
{
  // Initialize the event loop for window management
  let event_loop = winit::event_loop::EventLoop::new().unwrap();

  // Create a window with default attributes, title, and size
  let window = event_loop.create_window
  (
    winit::window::WindowAttributes::default()
    .with_title( "Rive demo" )
    .with_inner_size( winit::dpi::LogicalSize::new( 500, 500 ) )
  )
  .unwrap();

  // Initialize the Vello rendering context
  let mut render_context = vello::util::RenderContext::new().unwrap();
  let size = window.inner_size();

  // Create a surface for rendering, blocking until completion
  let surface = pollster::block_on
  (
    render_context.create_surface( &window, size.width, size.height )
  )
  .unwrap();

  // Initialize the Vello renderer with specific options
  let vello_renderer = vello::Renderer::new
  (
    &render_context.devices[ surface.dev_id ].device,
    &vello::RendererOptions
    {
      surface_format: Some( surface.format ),
      timestamp_period: 0.0,
      use_cpu: false,
    }
  )
  .unwrap();

  // Load and instantiate the Rive animation file
  let rive_file = rive_rs::File::new( include_bytes!( "../assets/demo.riv" ) ).unwrap();
  let artboard = rive_rs::Artboard::instantiate( &rive_file, rive_rs::Handle::Default ).unwrap();

  // Create the application instance with all necessary components
  let mut application = Application
  {
    viewport: rive_rs::Viewport::default(),
    scene: Box::< dyn rive_rs::Scene >::instantiate( &artboard, rive_rs::Handle::Default ).unwrap(),
    window,
    surface,
    render_context,
    vello_renderer,
  };

  // Start the event loop
  event_loop.run_app( &mut application ).unwrap();
}

/// Main application struct that holds all necessary components for rendering
/// and managing the Rive animation.
struct Application
{
  /// Viewport defining the visible area of the animation
  viewport : rive_rs::Viewport,

  /// The Rive scene containing the animation
  scene : Box< dyn rive_rs::Scene >,

  /// Window handle for the application
  window : winit::window::Window,

  /// Surface for rendering
  surface : vello::util::RenderSurface,

  /// Vello rendering context
  render_context : vello::util::RenderContext,

  /// Vello renderer instance for rendering scene to surface
  vello_renderer : vello::Renderer,
}

/// Implementation of the ApplicationHandler trait for handling window and application events
impl winit::application::ApplicationHandler for Application
{
  /// Called when the application is resumed
  fn resumed( &mut self, _event_loop : &winit::event_loop::ActiveEventLoop ) {}

  /// Handles various window events including redraw requests, resizing, cursor movement,
  /// and close requests etc.
  fn window_event
  (
    &mut self,
    event_loop : &winit::event_loop::ActiveEventLoop,
    _window_id : winit::window::WindowId,
    event : winit::event::WindowEvent,
  )
  {
    match event
    {
      // Handle redraw requests
      winit::event::WindowEvent::RedrawRequested =>
      {
        let device_handle = &self.render_context.devices[ self.surface.dev_id ];

        if let Ok( surface_texture ) = self.surface.surface.get_current_texture()
        {
          // Set up rendering components
          let mut rive_renderer = rive_rs::Renderer::default();
          let mut vello_scene = vello::Scene::default();
          let mut builder = vello::SceneBuilder::for_scene( &mut vello_scene );

          // Advance the animation and draw the current frame
          self.scene.advance_and_maybe_draw
          (
            &mut rive_renderer,
            std::time::Duration::from_secs_f64( 1.0 / 60.0 ),
            &mut self.viewport,
          );
          builder.append( rive_renderer.scene(), None );

          // Render the scene to the surface
          let _ = vello::block_on_wgpu
          (
            &device_handle.device,
            self.vello_renderer.render_to_surface_async
            (
              &device_handle.device,
              &device_handle.queue,
              &vello_scene,
              &surface_texture,
              &vello::RenderParams
              {
                base_color: vello::peniko::Color::WHITE,
                width: self.surface.config.width,
                height: self.surface.config.height,
              }
            )
          );

          // Present the texture to surface
          surface_texture.present();
        }

        device_handle.device.poll( wgpu::Maintain::Poll );
      },
      // Handle window resizing
      winit::event::WindowEvent::Resized( size ) =>
      {
        self.viewport.resize( size.width, size.height );
        self.render_context.resize_surface( &mut self.surface, size.width, size.height );
        self.window.request_redraw();
      },
      // Handle cursor movement for interactivity
      winit::event::WindowEvent::CursorMoved { position, .. } =>
      {
        self.scene.pointer_move( position.x as f32, position.y as f32, &self.viewport );
      },
      // Handle window close requests
      winit::event::WindowEvent::CloseRequested =>
      {
        event_loop.exit();
      },
      _ => {}
    }
  }

  /// Called when the event loop is about to wait for events
  /// Requests a redraw to maintain animation
  fn about_to_wait( &mut self, _event_loop : &winit::event_loop::ActiveEventLoop )
  {
    self.window.request_redraw();
  }
}
