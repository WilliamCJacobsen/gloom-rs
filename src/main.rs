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
pub mod mesh;
pub mod scene_graph;
pub mod object;
pub mod VAO;
pub mod toolbox;
pub mod animate;
// -------------------------------

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



/*Key pressed helper function*/

unsafe fn draw_scene(root: &scene_graph::SceneNode, view_projection_matrix: &glm::Mat4) {
    // Check if node is drawable, set uniforms, draw.
    
    if root.index_count > -1 {
        // checking if the indexcount is greater than -1, if it is then it contains items that will be drawn. 
        
        gl::BindVertexArray(root.vao_id);
        gl::UniformMatrix4fv(3, 1 as GLsizei, gl::FALSE, (view_projection_matrix * root.current_transformation_matrix).as_ptr()); // sending in the MVP - matrix
        gl::UniformMatrix4fv(4, 1 as GLsizei, gl::FALSE, (root.current_transformation_matrix).as_ptr()); // sending in the model matrix
        gl::DrawElements(
            gl::TRIANGLES, 
            3 * root.index_count,
            gl::UNSIGNED_INT, 
            ptr::null()
        ); 
    }

    // checking if the root has no more children. 
    if root.children.len() == 0 {
        return;
    }

    // Recurse
    for &child in &root.children {
        draw_scene(&*child, view_projection_matrix);
    }
}

unsafe fn update_node_transformations(root: &mut scene_graph::SceneNode, transformation_so_far: &glm::Mat4) {
    // Construct the correct transformation matrix

    // Update the node's transformation matrix
    // nodes location? what should i do with this information...
    //  *glm::rotation(0.2, &root.rotation) *  );
    let mut rotate_item: glm::Mat4 = glm::identity();

    if root.rotation.x != 0.0 {
        rotate_item *= glm::rotation(root.rotation.x, &glm::vec3(1.0, 0.0 , 0.0)) 
    }

    if root.rotation.y != 0.0 {
        rotate_item *= glm::rotation(root.rotation.y, &glm::vec3(0.0, 1.0 , 0.0)) 
    }

    if root.rotation.z != 0.0 {
        rotate_item *= glm::rotation(root.rotation.z, &glm::vec3(0.0, 0.0 , 1.0)) 
    }


    rotate_item = glm::translation(&root.reference_point) * rotate_item * glm::translation(&glm::vec3(-root.reference_point.x, -root.reference_point.y, -root.reference_point.z));
    
    
    root.current_transformation_matrix =  transformation_so_far * glm::translation(&root.position) * rotate_item ;

    
    // checking if the root has no more children. 
    if root.children.len() == 0 {
        return;
    }

    for &child in &root.children {// Recurse
        update_node_transformations(&mut *child,
        &root.current_transformation_matrix);
    }
}

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

        let mut place_holder_vao: gl::types::GLuint;


        let mut matrix_integer: GLint;

        let fov : f32 = std::f32::consts::PI/4.0;

        
        /* generating the Terrain mesh */
        let terrain_mesh: mesh::Mesh = mesh::Terrain::load("./resources/lunarsurface.obj");

        /*loading all messhes from helicopter*/
        let helicopter: mesh::Helicopter = mesh::Helicopter::load("./resources/helicopter.obj");
        let mut scene_graph_obj = scene_graph::SceneNode::new();
        // == // Set up your VAO here
        
        let mut zombie_heilcopter1 :std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>;
        let mut zombie_heilcopter2 : std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>;
        let mut zombie_heilcopter3 : std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>;
        let mut zombie_heilcopter4 : std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>> ;
        let mut zombie_heilcopter5 : std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>;
        let mut helicopter_object : std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>;
        unsafe {
            // 

            let mut terrain_obj = object::new_terrain(&mut vao, &terrain_mesh);

            helicopter_object = object::new_helicopter(&mut vao, &helicopter);

             zombie_heilcopter1 = object::zombie_helicopter(&mut vao, &helicopter);
             zombie_heilcopter2 = object::zombie_helicopter(&mut vao, &helicopter);
             zombie_heilcopter3 = object::zombie_helicopter(&mut vao, &helicopter);
             zombie_heilcopter4 = object::zombie_helicopter(&mut vao, &helicopter);
             zombie_heilcopter5 = object::zombie_helicopter(&mut vao, &helicopter);
            // Adding helicopter graph tree to scene_graph_obj and the one terrain_mesh object node.

            terrain_obj.add_child(&helicopter_object);
            terrain_obj.add_child(&zombie_heilcopter1);
            terrain_obj.add_child(&zombie_heilcopter2);
            terrain_obj.add_child(&zombie_heilcopter3);
            terrain_obj.add_child(&zombie_heilcopter4);
            terrain_obj.add_child(&zombie_heilcopter5);
            
            scene_graph_obj.add_child(&terrain_obj);
            
            
            scene_graph_obj.print();




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

        let theta : f32 = 10.0;
        let camera_speed : f32 = 30.0;

        /* Create a camera struct to handle the camera movements. */
        let mut camera_struct = unsafe{  camera::Camera::new((SCREEN_H as f32)/(SCREEN_W as f32), fov , 1.0, 1000.0, -28.0) };
        
     
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
                            unsafe{ 
                                camera_struct.forward_backward(new_camera_speed);
                                animate::move_body(&mut helicopter_object, glm::vec3(0.0, 0.0, -camera_speed), delta_time);
                             }

                        },
                        VirtualKeyCode::S => {
                            unsafe{
                                 camera_struct.forward_backward(-new_camera_speed);
                                 animate::move_body(&mut helicopter_object, glm::vec3(0.0, 0.0, camera_speed), delta_time);
                                 }
  
                        },
                        VirtualKeyCode::A => {
                            unsafe{ 
                                camera_struct.left_right(new_camera_speed);
        
                             }

                        },
                        VirtualKeyCode::D => {
                            unsafe{ 
                                camera_struct.left_right(-new_camera_speed);
                                animate::move_body(&mut helicopter_object, glm::vec3(camera_speed, 0.0, 0.0), delta_time);
                             }

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

                // Animating the zombie helicopters here
                animate::animate(&mut zombie_heilcopter5, elapsed, delta_time, 0.0);
                animate::animate(&mut zombie_heilcopter4, elapsed, delta_time, 1.0);
                animate::animate(&mut zombie_heilcopter3, elapsed, delta_time, 2.0);
                animate::animate(&mut zombie_heilcopter2, elapsed, delta_time, 3.0);
                animate::animate(&mut zombie_heilcopter1, elapsed, delta_time, 4.5);
                // Done animating the zombie helicopters...

                animate::animate(&mut helicopter_object, elapsed, delta_time, 4.0);
                update_node_transformations(&mut scene_graph_obj, &glm::identity());
                draw_scene(&scene_graph_obj, &camera_struct.move_camera_matrix());

            
                
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
