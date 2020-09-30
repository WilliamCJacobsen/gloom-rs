use crate::VAO;
use crate::scene_graph;
use crate::mesh;
use crate::toolbox;
extern crate nalgebra_glm as glm;



pub unsafe fn new_helicopter(vao : &mut u32, mesh: &mesh::Helicopter) -> std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>> {
    //loading in the helicopter
    
    let mut place_holder_vao = VAO::vertex_array_object( vao, &mesh.body.vertices, &mesh.body.indices, &mesh.body.colors, &mesh.body.normals);
    let mut helicopter_object = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.body.index_count,String::from("body") );

    place_holder_vao = VAO::vertex_array_object( vao, &mesh.main_rotor.vertices, &mesh.main_rotor.indices, &mesh.main_rotor.colors, &mesh.main_rotor.normals);
    let mut main_rotor = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.main_rotor.index_count, String::from("main"));
    main_rotor.rotation = glm::vec3(0.0, 1.0, 0.0);

    helicopter_object.add_child(&main_rotor);

    place_holder_vao = VAO::vertex_array_object(vao, &mesh.tail_rotor.vertices, &mesh.tail_rotor.indices, &mesh.tail_rotor.colors, &mesh.tail_rotor.normals);
    let mut tail_obj = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.tail_rotor.index_count, String::from("tail"));
    tail_obj.reference_point = glm::vec3(0.35, 2.3, 10.4);
    tail_obj.rotation = glm::vec3(1.0, 0.0, 0.0);

    helicopter_object.add_child(&tail_obj);

    place_holder_vao = VAO::vertex_array_object( vao, &mesh.door.vertices, &mesh.door.indices, &mesh.door.colors, &mesh.door.normals);
    helicopter_object.add_child(&scene_graph::SceneNode::from_vao(place_holder_vao, mesh.door.index_count, String::from("door")));

    helicopter_object
}

pub unsafe fn zombie_helicopter(vao : &mut u32, mesh: &mesh::Helicopter) -> std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>> {
    //loading in the helicopter
    
    let mut place_holder_vao = VAO::vertex_array_object( vao, &mesh.body.vertices, &mesh.body.indices, &mesh.body.colors, &mesh.body.normals);
    let mut helicopter_object = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.body.index_count,String::from("zombie") );

    place_holder_vao = VAO::vertex_array_object( vao, &mesh.main_rotor.vertices, &mesh.main_rotor.indices, &mesh.main_rotor.colors, &mesh.main_rotor.normals);
    let mut main_rotor = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.main_rotor.index_count, String::from("main"));
    main_rotor.rotation = glm::vec3(0.0, 1.0, 0.0);

    helicopter_object.add_child(&main_rotor);

    place_holder_vao = VAO::vertex_array_object(vao, &mesh.tail_rotor.vertices, &mesh.tail_rotor.indices, &mesh.tail_rotor.colors, &mesh.tail_rotor.normals);
    let mut tail_obj = scene_graph::SceneNode::from_vao(place_holder_vao, mesh.tail_rotor.index_count, String::from("tail"));
    tail_obj.reference_point = glm::vec3(0.35, 2.3, 10.4);


    helicopter_object.add_child(&tail_obj);

    place_holder_vao = VAO::vertex_array_object( vao, &mesh.door.vertices, &mesh.door.indices, &mesh.door.colors, &mesh.door.normals);
    helicopter_object.add_child(&scene_graph::SceneNode::from_vao(place_holder_vao, mesh.door.index_count, String::from("door")));

    helicopter_object

}

pub unsafe fn new_terrain(vao : &mut u32, mesh: &mesh::Mesh) -> std::mem::ManuallyDrop<std::pin::Pin<std::boxed::Box<scene_graph::SceneNode>>>{

    let place_holder_vao = VAO::vertex_array_object(vao, &mesh.vertices, &mesh.indices, &mesh.colors, &mesh.normals);

    return scene_graph::SceneNode::from_vao(place_holder_vao, mesh.index_count, String::from("terrain"));
  
}


