#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
in vec3 newColor;

uniform sampler2D texture_diffuse1;
uniform float mixValue;

void main()
{
    vec4 colorView = vec4(newColor, 1.0f);
    vec4 texturedView = texture(texture_diffuse1, TexCoords);
    FragColor = mix(colorView, texturedView, mixValue);
}
