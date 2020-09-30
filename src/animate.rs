use crate::toolbox;
use crate::scene_graph;

unsafe fn tail(tail_rotor : &mut scene_graph::SceneNode, delta_time : f32)
{
    tail_rotor.rotation += delta_time * glm::vec3(-15.0, 0.0, 0.0);
}

unsafe fn main( main_rotor: &mut scene_graph::SceneNode, delta_time : f32)
{
    main_rotor.rotation += delta_time * glm::vec3(0.0, 10.0, 0.0);
}

unsafe fn zombie(body : &mut scene_graph::SceneNode, total_time: f32, offset: f32)
{
    let heading = toolbox::simple_heading_animation(total_time + offset);

    body.position = glm::vec3(heading.x, body.position.y, heading.z);

    body.rotation = glm::vec3(
        heading.pitch,
        heading.yaw,
        heading.roll
    );
}

pub unsafe fn animate(root :  &mut scene_graph::SceneNode, total_time : f32, delta_time:f32, offset: f32)
{
    match root.name.as_str() {

        "main" => main(root, delta_time),
        "tail" => tail(root, delta_time),
        "zombie" => zombie(root, total_time, offset),
        _ => ()
    }
    

    // Recurse
    for &child in &root.children {
        animate(&mut *child, total_time, delta_time, offset);
    }
}