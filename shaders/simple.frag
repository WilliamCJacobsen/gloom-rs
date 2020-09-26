#version 430 core

out vec4 color;


in VS_OUTPUT {
    vec4 color;
} IN;


void main()
{   
    color = IN.color;
    
}