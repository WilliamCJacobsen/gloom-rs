#version 430 core

out vec4 color;


in VS_OUTPUT {
    vec4 color;
    vec3 normal;
} IN;


vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{   
    color =  vec4(vec3(IN.color) *  max(dot(IN.normal, -lightDirection), 0.0), IN.color[3]);
}