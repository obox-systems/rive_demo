use rive_rs::Instantiate;

fn main()
{
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  let window = event_loop.create_window
  (
    winit::window::WindowAttributes::default()
    .with_title( "Rive demo" )
    .with_inner_size( winit::dpi::LogicalSize::new( 500, 500 ) )
  )
  .unwrap();
  let mut render_context = vello::util::RenderContext::new().unwrap();
  let size = window.inner_size();
  let surface = pollster::block_on
  (
    render_context.create_surface( &window, size.width, size.height )
  )
  .unwrap();
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
  let rive_file = rive_rs::File::new( include_bytes!( "../assets/demo.riv" ) ).unwrap();
  let artboard = rive_rs::Artboard::instantiate( &rive_file, rive_rs::Handle::Default ).unwrap();
  let mut application = Application
  {
    viewport: rive_rs::Viewport::default(),
    scene: Box::< dyn rive_rs::Scene >::instantiate( &artboard, rive_rs::Handle::Default ).unwrap(),
    window,
    surface,
    render_context,
    vello_renderer,
  };

  event_loop.run_app( &mut application ).unwrap();
}

struct Application
{
  viewport : rive_rs::Viewport,
  scene : Box< dyn rive_rs::Scene >,
  window : winit::window::Window,
  surface : vello::util::RenderSurface,
  render_context : vello::util::RenderContext,
  vello_renderer : vello::Renderer,
}

impl winit::application::ApplicationHandler for Application
{
  fn resumed( &mut self, _event_loop : &winit::event_loop::ActiveEventLoop ) {}

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
      winit::event::WindowEvent::RedrawRequested =>
      {
        let device_handle = &self.render_context.devices[ self.surface.dev_id ];

        if let Ok( surface_texture ) = self.surface.surface.get_current_texture()
        {
          let mut rive_renderer = rive_rs::Renderer::default();
          let mut vello_scene = vello::Scene::default();
          let mut builder = vello::SceneBuilder::for_scene( &mut vello_scene );

          self.scene.advance_and_maybe_draw
          (
            &mut rive_renderer,
            std::time::Duration::from_secs_f64( 1.0 / 60.0 ),
            &mut self.viewport,
          );
          builder.append( rive_renderer.scene(), None );

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

          surface_texture.present();
        }

        device_handle.device.poll( wgpu::Maintain::Poll );
      },
      winit::event::WindowEvent::Resized( size ) =>
      {
        self.viewport.resize( size.width, size.height );
        self.render_context.resize_surface( &mut self.surface, size.width, size.height );
        self.window.request_redraw();
      },
      winit::event::WindowEvent::CursorMoved { position, .. } =>
      {
        self.scene.pointer_move( position.x as f32, position.y as f32, &self.viewport );
      },
      winit::event::WindowEvent::CloseRequested =>
      {
        event_loop.exit();
      },
      _ => {}
    }
  }

  fn about_to_wait( &mut self, _event_loop : &winit::event_loop::ActiveEventLoop )
  {
    self.window.request_redraw();
  }
}
