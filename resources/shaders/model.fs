#version 330 core
out vec4 FragColor;

in vec2 TexCoords;
in vec3 newColor;
in vec3 ObjPos;

uniform sampler2D texture_diffuse1;
uniform float mixValue;
uniform int useGeneratedMapping;
uniform float generatedTexScale;

vec3 safe_normalize(vec3 value)
{
    float len = length(value);
    if (len < 0.00001) {
        return vec3(0.0, 1.0, 0.0);
    }
    return value / len;
}

vec4 sample_triplanar(vec3 object_pos, float tex_scale)
{
    vec3 dpdx = dFdx(object_pos);
    vec3 dpdy = dFdy(object_pos);
    vec3 surface_normal = safe_normalize(cross(dpdx, dpdy));

    vec3 weights = pow(abs(surface_normal), vec3(4.0));
    float weight_sum = weights.x + weights.y + weights.z;
    if (weight_sum < 0.00001) {
        weights = vec3(0.0, 1.0, 0.0);
    } else {
        weights /= weight_sum;
    }

    vec2 uv_x = object_pos.yz * tex_scale;
    vec2 uv_y = object_pos.xz * tex_scale;
    vec2 uv_z = object_pos.xy * tex_scale;

    if (surface_normal.x < 0.0) {
        uv_x.x = -uv_x.x;
    }
    if (surface_normal.y < 0.0) {
        uv_y.x = -uv_y.x;
    }
    if (surface_normal.z < 0.0) {
        uv_z.x = -uv_z.x;
    }

    vec4 sample_x = texture(texture_diffuse1, uv_x);
    vec4 sample_y = texture(texture_diffuse1, uv_y);
    vec4 sample_z = texture(texture_diffuse1, uv_z);

    return sample_x * weights.x + sample_y * weights.y + sample_z * weights.z;
}

void main()
{
    vec4 colorView = vec4(newColor, 1.0);

    vec4 texturedView;
    if (useGeneratedMapping == 1) {
        texturedView = sample_triplanar(ObjPos, generatedTexScale);
    } else {
        texturedView = texture(texture_diffuse1, TexCoords);
    }

    FragColor = mix(colorView, texturedView, mixValue);
}
