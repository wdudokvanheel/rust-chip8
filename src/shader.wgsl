struct DisplayUniform {
    values: array<vec4<u32>, 512>,
};

@group(0) @binding(0)
var<uniform> display: DisplayUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// Vertex shader

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let x = floor((in.clip_position.x / 1280.0) * 64.0);
	let y = floor((in.clip_position.y / 640.0) * 32.0);

	var index : u32 = u32(((y * 64.0) + x));

	let d = display.values[index / u32(4)];
	let i = index % u32(4);

    if (d[i] > u32(0) ){
        return vec4<f32>(0.598,0.38,0.833, 1.0);
    }else{
        return vec4<f32>(0.014,0.022,0.029, 1.0);
    }
}
