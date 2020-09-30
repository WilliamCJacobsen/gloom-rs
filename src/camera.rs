use gl;
use std::{
    ptr,
    str,
    ffi::CString,
    path::Path,
};
/**
 * AUTHOR : WILLIAM CHAKROUN JACOBSEN
 * 
 * Note for teacher:
 * 
 * This file acts as a simulator for the camrea. 
 * This handles all the proparites of the camera and how it acts.
 * 
 * 
 * 
*/

//-------------- Struct --------------
//init a public struct called camera.
pub struct Camera {
    pub m_perspective: glm::Mat4,
    m_rotation :  Box<glm::Mat4>, // the variabels which holds the rotation matrix
    m_translation :  Box<glm::Mat4> // the variable which holds the translation matrix.
}

//-------------------------------------


impl Camera{
    // create a new struct called camera.
    pub unsafe fn new(aspect : f32, fov : f32, z_near : f32, z_far: f32, offset: f32) -> Camera {
        Camera { 
            m_perspective: glm::perspective(aspect, fov, z_near, z_far), // create the perspective matrix for task 4 [report]
            m_rotation:  Box::new(glm::identity()), // initalizing the rotation mattrix with an identity matrix.
            m_translation:  Box::new(translate(0.0, -9.0, offset)) // init the translation matrix with an offset. ( comon is -5.0 f32)
         }
    }

    // taking in a f32 value as parameter and adds it to the current translation matrix.
    pub unsafe fn forward_backward(&mut self, value : f32){
        *self.m_translation.as_mut() *= translate(0.0, 0.0, value)
    }

    // taking in a f32 value as parameter and adds it to the current translation matrix.
    pub unsafe fn up_down(&mut self, value : f32){
        *self.m_translation.as_mut() *= translate(0.0, value, 0.0)
    }

    // taking in a f32 value as parameter and adds it to the current translation matrix.
    pub unsafe fn left_right(&mut self, value : f32){
        *self.m_translation.as_mut() *= translate( value, 0.0 ,0.0)

    }

    // multiplying the rotation matrix R_x to the current rotation matrix.
    pub unsafe fn yaw(&mut self, theta: f32){
        *self.m_rotation.as_mut() *= rotate_x(theta)
    }

    // reset the rotation matrix to the center of the screen. 
    pub unsafe fn reset_rotation(&mut self){
        *self.m_rotation.as_mut() = glm::identity();
    }

    // multiplying the rotation matrix R_y to the current rotation matrix.
    pub unsafe fn pitch(&mut self, theta: f32){
        *self.m_rotation.as_mut() *= rotate_y(theta)
    }

    // multiplying all the matrixes in the correct order. Which is translation, then rotation and then the perspective.
    pub unsafe fn move_camera_matrix(&self) -> glm::Mat4 {
       return  self.m_perspective * self.m_rotation.as_ref() * self.m_translation.as_ref() 
    }

}

// ---------------------- Helper functions ----------------------
// created own helper matrices instead of using glms functions. This is to get a better understanding of how the transformations work.

// helper funciton for the rotation matrix R_x.
unsafe fn rotate_x(theta: f32) -> glm::Mat4{
    return glm::mat4x4(
        f32::cos(theta), 0.0,f32::sin(theta), 0.0,
        0.0, 1.0, 0.0, 0.0,
        -f32::sin(theta), 0.0, f32::cos(theta), 0.0,
        0.0, 0.0, 0.0, 1.0
    );
}

// helper function for the rotation matrix R_y
unsafe fn rotate_y(theta: f32) -> glm::Mat4{
    return glm::mat4x4(
        1.0, 0.0, 0.0, 0.0,
        0.0, f32::cos(theta), -f32::sin(theta), 0.0,
        0.0, f32::sin(theta), f32::cos(theta), 0.0,
        0.0, 0.0, 0.0, 1.0)
}

// helper function to translate a matrix.
unsafe fn translate(x: f32, y :f32, z : f32) -> glm::Mat4{
   return glm::mat4x4(
       1.0, 0.0, 0.0, x,
       0.0, 1.0, 0.0, y,
       0.0, 0.0, 1.0, z,
       0.0, 0.0, 0.0, 1.0
   )

}
// ---------------------------------------------------------------------