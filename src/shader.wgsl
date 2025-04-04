struct DisplayUniform {
    values: array<vec4<u32>, 512>,
    width: f32,
    height: f32,
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
	let posX = in.clip_position.x - 0.5;
	let posY = in.clip_position.y - 0.5;
	let x = ((posX / display.width ) * 64.0);
	let y = ((posY / display.height) * 32.0);

	var index : u32 = u32(((floor(y) * 64.0) + floor(x)));

	let d = display.values[index / u32(4)];
	let i = index % u32(4);

	var finalColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    if (d[i] > u32(0) ){
        finalColor = vec4<f32>(0.427, 0.176, 0.969, 1.0);
    }else{
        finalColor = vec4<f32>(0.00, 0.00, 0.00, 1.0);
    }

    return vec4(pow(finalColor.rgb, vec3(2.2)), finalColor.a);;
}
