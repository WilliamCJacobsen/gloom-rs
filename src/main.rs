extern crate nalgebra_glm as glm;
use gl::types::*;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod camera;

use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 600;
const SCREEN_H: u32 = 500;

// --- last assignment import ---
mod mesh;
// -------------------------------

unsafe fn buffer<T>(buffer_id: &mut u32, items: &Vec<T>, buffer_type: gl::types::GLenum, location_id: u32, n_values: i32) -> () {
    gl::GenBuffers(1, buffer_id);
    gl::BindBuffer(buffer_type, *buffer_id);

    gl::BufferData(
        buffer_type,
        (items.len() * std::mem::size_of::<T>()) as isize, 
        pointer_to_array(items),
        gl::STATIC_DRAW
    );

    if buffer_type == gl::ARRAY_BUFFER {
        gl::VertexAttribPointer(
            location_id,
            n_values,
            gl::FLOAT,
            gl::FALSE,
            (std::mem::size_of::<T>() as i32) * n_values, /* The size is mem * x to the next x*/
            ptr::null()
        );
        gl::EnableVertexAttribArray(location_id);
    }


}

// helper function to bind vectors to bind vectors to VAO objects. 
unsafe fn bind_buffers(buffer_id: &mut u32, items: &Vec<f32>, colors: &Vec<f32>, normals :&Vec<f32>, buffer_type: gl::types::GLenum ) -> () { 
    let positions: u32 = 0; // location id for the buffer object positions.
    let colors_id: u32 = 1; // location id for the buffer object colors (rgba)
    let normal_id: u32 = 15; // location id for the buffer normals


    buffer(buffer_id, items, buffer_type, positions, 3);
    buffer(buffer_id, colors, buffer_type, colors_id, 4);
    
}

unsafe fn vertex_array_object(voc_id: &mut u32, vertex: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals : &Vec<f32>) -> u32 {
    let mut vao_id: u32 = *voc_id; // borrows the global id to get an individual id.

    
    gl::GenVertexArrays(1,  &mut vao_id); // generating the VAO.
    gl::BindVertexArray(vao_id);

    *voc_id = *voc_id + 1; /* adding 1 to the value voc_id */

    let mut buffer_id = 0;

    bind_buffers(&mut buffer_id, vertex, colors, normals , gl::ARRAY_BUFFER); //binding vertexes and colors to Vertex buffer objects. 
    buffer(&mut buffer_id, indices, gl::ELEMENT_ARRAY_BUFFER, 0, 0);


    vao_id
}
// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

/*Key pressed helper function*/

// == // Modify and complete the function below for the first task
// unsafe fn FUNCTION_NAME(ARGUMENT_NAME: &Vec<f32>, ARGUMENT_NAME: &Vec<u32>) -> u32 { } 

fn main() {
    // Set up the necessary objects to deal with windows and event handling

    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);
    
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO here
        let mut vao: gl::types::GLuint = 0;


        let mut matrix_integer: GLint;

        let fov : f32 = std::f32::consts::PI/4.0;

        let mut vertices_three_triangle: Vec<f32> = vec![
            -0.2, 0.0, 0.0, 
             0.2, 0.0, 0.0,  
             0.0, 0.3, 0.0,

             0.2, 0.0, 0.0, 
             0.6, 0.0, 0.0,  
             0.4, 0.3, 0.0,

             -0.6, 0.0, 0.0, 
             -0.2, 0.0, 0.0,  
             -0.4, 0.3, 0.0,
        ];

        let mut indices_three_triangle = vec![
            0,1,2,
            3,4,5,
            6,7,8
        ];

        let mut colors = vec![
            1.0,0.0,1.0,1.0,
            0.0,0.0,1.0,1.0,
            0.0,1.0,0.0,1.0,

            1.0,0.0,1.0,1.0,
            0.0,0.0,1.0,1.0,
            0.0,1.0,0.0,1.0,

            1.0,0.0,1.0,1.0,
            0.0,0.0,1.0,1.0,
            0.0,1.0,0.0,1.0
        ];

        
        /* generating the Terrain mesh */
        let terrain_mesh: mesh::Mesh = mesh::Terrain::load("./resources/lunarsurface.obj");


        // == // Set up your VAO here
        unsafe {

            // [report task 1] Added colors to the VAO. 
            vertex_array_object(&mut vao, &terrain_mesh.vertices, &terrain_mesh.indices, &terrain_mesh.colors, &terrain_mesh.normals);


            // Basic usage of shader helper
            // The code below returns a shader object, which contains the field .program_id
            // The snippet is not enough to do the assignment, and will need to be modified (outside of just using the correct path)
            let shader_builder = shader::ShaderBuilder::new().attach_file("./shaders/simple.frag").attach_file("./shaders/simple.vert").link();

            let program_id = shader_builder.program_id; // fetching the program id

            gl::UseProgram(program_id);
            
            // [report task 3] fetching the location identity for the transformation matrix in Vertex shader.
            matrix_integer = gl::GetUniformLocation(program_id, "transformation_matrix\0".as_ptr() as *const i8);
        }


        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loops

        let theta : f32 = 30.0;
        let camera_speed : f32 = 3.0;

        /* Create a camera struct to handle the camera movements. */
        let mut camera_struct = unsafe{  camera::Camera::new((SCREEN_H as f32)/(SCREEN_W as f32), fov , 1.0, 100.0, -5.0) };


        
     
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            let new_theta = theta * delta_time; // taking the delta time and multiplying it to the theta
            let new_camera_speed = camera_speed * delta_time;  // taking the delta time and multiplying it to camera_speed. So Frames per second does not play a part in the movement speed.

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {

                for key in keys.iter() {
                    // For each key pressed the camera will be affected and new matrices will be calculated. 

                    match key {
                        VirtualKeyCode::W => {
                            unsafe{ camera_struct.forward_backward(new_camera_speed) }

                        },
                        VirtualKeyCode::S => {
                            unsafe{ camera_struct.forward_backward(-new_camera_speed) }
  
                        },
                        VirtualKeyCode::A => {
                            unsafe{ camera_struct.left_right(-new_camera_speed) }

                        },
                        VirtualKeyCode::D => {
                            unsafe{ camera_struct.left_right(new_camera_speed) }

                        },
                        VirtualKeyCode::Q => {
                            // When Q is pressed the camera goes up
                            unsafe{ camera_struct.up_down(new_camera_speed) }

                        },
                        VirtualKeyCode::E=> {
                            // When E is pressed the camera goes down
                            unsafe{ camera_struct.up_down(-new_camera_speed) }

                        },
                        VirtualKeyCode::Left => {
                            unsafe{ camera_struct.yaw(-new_theta) }

                        },
                        VirtualKeyCode::Right => {
                            unsafe{ camera_struct.yaw(new_theta) }
                        },
                        VirtualKeyCode::Up => {
                            unsafe{ camera_struct.pitch(-new_theta) }
                        },
                        VirtualKeyCode::Down => {
                            unsafe{ camera_struct.pitch(new_theta) }
                        },
                        VirtualKeyCode::R => {
                            // I wanted to reset the camera rotation back to starting position. 
                            unsafe{ camera_struct.reset_rotation() }
                        },
                        _ => { }
                    }
                } 
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {



                *delta = (0.0, 0.0);
            }

    

            unsafe {



                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                // Issue the necessary commands to draw your scene here

                gl::DrawElements(
                    gl::TRIANGLES, 
                    3 * terrain_mesh.index_count, 
                    gl::UNSIGNED_INT, 
                    ptr::null()
                ); 
                

        
                // takes in the matrix Identity for the vertex shader and sends in the product from camera struct.
                gl::UniformMatrix4fv(matrix_integer, 1 as GLsizei, gl::FALSE, camera_struct.move_camera_matrix().as_ptr()); // [report task 3]
            
                
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
