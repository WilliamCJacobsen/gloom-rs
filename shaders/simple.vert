#version 430 core

layout(location = 0) in vec3 position;

layout(location = 1) in vec4 color;

layout(location = 2) in vec3 normal;

layout(location = 3) uniform mat4 transformation_matrix;

layout(location = 4) uniform mat4 model_matrix;

out VS_OUTPUT{
    vec4 color;
    vec3 normal;
} OUT;



void main()
{
    gl_Position = transformation_matrix * vec4(position, 1.0f);
    OUT.color = color;
    OUT.normal = normalize(mat3(model_matrix) * normal);
}