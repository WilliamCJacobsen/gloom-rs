// created this file so it is not as much in the main as it already is.
use gl;
use std::{ mem, ptr, os::raw::c_void };


pub unsafe fn buffer<T>(buffer_id: &mut u32, items: &Vec<T>, buffer_type: gl::types::GLenum, location_id: u32, n_values: i32) -> () {
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
pub unsafe fn bind_buffers(buffer_id: &mut u32, items: &Vec<f32>, colors: &Vec<f32>, normals :&Vec<f32>, buffer_type: gl::types::GLenum ) -> () { 
    let positions: u32 = 0; // location id for the buffer object positions.
    let colors_id: u32 = 1; // location id for the buffer object colors (rgba)
    let normal_id: u32 = 2; // location id for the buffer normals


    buffer(buffer_id, items, buffer_type, positions, 3);
    buffer(buffer_id, colors, buffer_type, colors_id, 4);
    buffer(buffer_id, normals, buffer_type, normal_id, 3);
    
}

pub unsafe fn vertex_array_object(voc_id: &mut u32, vertex: &Vec<f32>, indices: &Vec<u32>, colors: &Vec<f32>, normals : &Vec<f32>) -> u32 {
    let mut vao_id: u32 = *voc_id; // borrows the global id to get an individual id.

    *voc_id = *voc_id + 1; /* adding 1 to the value voc_id */
 
    gl::GenVertexArrays(1,  &mut vao_id); // generating the VAO.
    gl::BindVertexArray(vao_id);

    

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
