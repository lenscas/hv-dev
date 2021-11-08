#version 300 es

in mediump vec3 a_Pos;
in mediump vec4 a_VertColor;

uniform mediump mat4 u_Tx;
uniform mediump vec4 u_Color;
uniform mediump mat4 u_MVP;

out mediump vec4 v_Color;

void main() {
    v_Color = u_Color * a_VertColor;
    vec4 position = a_Tx * vec4(a_Pos, 1.0);

    gl_Position = u_MVP * position;
}