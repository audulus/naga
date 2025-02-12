fn main_1() {
    var a: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    var b: vec4<f32> = vec4<f32>(2.0, 2.0, 2.0, 2.0);
    var m: mat4x4<f32>;
    var i: i32 = 5;
    var ceilOut: vec4<f32>;
    var roundOut: vec4<f32>;
    var floorOut: vec4<f32>;
    var fractOut: vec4<f32>;
    var truncOut: vec4<f32>;
    var sinOut: vec4<f32>;
    var absOut: vec4<f32>;
    var sqrtOut: vec4<f32>;
    var inversesqrtOut: vec4<f32>;
    var expOut: vec4<f32>;
    var exp2Out: vec4<f32>;
    var signOut: vec4<f32>;
    var transposeOut: mat4x4<f32>;
    var normalizeOut: vec4<f32>;
    var sinhOut: vec4<f32>;
    var cosOut: vec4<f32>;
    var coshOut: vec4<f32>;
    var tanOut: vec4<f32>;
    var tanhOut: vec4<f32>;
    var acosOut: vec4<f32>;
    var asinOut: vec4<f32>;
    var logOut: vec4<f32>;
    var log2Out: vec4<f32>;
    var lengthOut: f32;
    var determinantOut: f32;
    var bitCountOut: i32;
    var bitfieldReverseOut: i32;
    var atanOut: f32;
    var atan2Out: f32;
    var modOut: f32;
    var powOut: vec4<f32>;
    var dotOut: f32;
    var maxOut: vec4<f32>;
    var minOut: vec4<f32>;
    var reflectOut: vec4<f32>;
    var crossOut: vec3<f32>;
    var outerProductOut: mat4x4<f32>;
    var distanceOut: f32;
    var stepOut: vec4<f32>;
    var rad: vec4<f32>;
    var deg: f32;

    let _e6 = a;
    let _e7 = b;
    let _e8 = a;
    let _e9 = b;
    m = mat4x4<f32>(vec4<f32>(_e6.x, _e6.y, _e6.z, _e6.w), vec4<f32>(_e7.x, _e7.y, _e7.z, _e7.w), vec4<f32>(_e8.x, _e8.y, _e8.z, _e8.w), vec4<f32>(_e9.x, _e9.y, _e9.z, _e9.w));
    let _e35 = a;
    ceilOut = ceil(_e35);
    let _e39 = a;
    roundOut = round(_e39);
    let _e43 = a;
    floorOut = floor(_e43);
    let _e47 = a;
    fractOut = fract(_e47);
    let _e51 = a;
    truncOut = trunc(_e51);
    let _e55 = a;
    sinOut = sin(_e55);
    let _e59 = a;
    absOut = abs(_e59);
    let _e63 = a;
    sqrtOut = sqrt(_e63);
    let _e67 = a;
    inversesqrtOut = inverseSqrt(_e67);
    let _e71 = a;
    expOut = exp(_e71);
    let _e75 = a;
    exp2Out = exp2(_e75);
    let _e79 = a;
    signOut = sign(_e79);
    let _e83 = m;
    transposeOut = transpose(_e83);
    let _e87 = a;
    normalizeOut = normalize(_e87);
    let _e91 = a;
    sinhOut = sinh(_e91);
    let _e95 = a;
    cosOut = cos(_e95);
    let _e99 = a;
    coshOut = cosh(_e99);
    let _e103 = a;
    tanOut = tan(_e103);
    let _e107 = a;
    tanhOut = tanh(_e107);
    let _e111 = a;
    acosOut = acos(_e111);
    let _e115 = a;
    asinOut = asin(_e115);
    let _e119 = a;
    logOut = log(_e119);
    let _e123 = a;
    log2Out = log2(_e123);
    let _e127 = a;
    lengthOut = length(_e127);
    let _e131 = m;
    determinantOut = determinant(_e131);
    let _e135 = i;
    bitCountOut = countOneBits(_e135);
    let _e139 = i;
    bitfieldReverseOut = reverseBits(_e139);
    let _e142 = a;
    let _e144 = a;
    atanOut = atan(_e144.x);
    let _e148 = a;
    let _e150 = a;
    let _e152 = a;
    let _e154 = a;
    atan2Out = atan2(_e152.x, _e154.y);
    let _e158 = a;
    let _e160 = b;
    let _e162 = a;
    let _e164 = b;
    modOut = (_e162.x % _e164.x);
    let _e170 = a;
    let _e171 = b;
    powOut = pow(_e170, _e171);
    let _e176 = a;
    let _e177 = b;
    dotOut = dot(_e176, _e177);
    let _e182 = a;
    let _e183 = b;
    maxOut = max(_e182, _e183);
    let _e188 = a;
    let _e189 = b;
    minOut = min(_e188, _e189);
    let _e194 = a;
    let _e195 = b;
    reflectOut = reflect(_e194, _e195);
    let _e198 = a;
    let _e200 = b;
    let _e202 = a;
    let _e204 = b;
    crossOut = cross(_e202.xyz, _e204.xyz);
    let _e210 = a;
    let _e211 = b;
    outerProductOut = outerProduct(_e210, _e211);
    let _e216 = a;
    let _e217 = b;
    distanceOut = distance(_e216, _e217);
    let _e222 = a;
    let _e223 = b;
    stepOut = step(_e222, _e223);
    let _e227 = a;
    rad = radians(_e227);
    let _e230 = a;
    let _e232 = a;
    deg = degrees(_e232.x);
    return;
}

@stage(vertex) 
fn main() {
    main_1();
    return;
}
