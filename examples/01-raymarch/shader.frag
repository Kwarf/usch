// The MIT License
// Copyright © 2018-2021 AJ Weeks
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// Part of the Raymarching workshop
// https://github.com/electricsquare/raymarching-workshop

#version 420

layout(std140, binding = 0)
uniform Uniforms {
	vec3 iResolution;
	float iTime;
};

layout(location = 0) out vec4 fragColor;

float sdSphere(vec3 p, float r)
{
	return length(p)-r;
}

float SDF(vec3 pos)
{
	float t = sdSphere(pos-vec3(0,0,10), 3.0);
	
	return t;
}

vec3 calcNormal(vec3 pos)
{
	// Center sample
	float c = SDF(pos);
	// Use offset samples to compute gradient / normal
	vec2 eps_zero = vec2(0.001, 0.0);
	return normalize(vec3(
		SDF(pos + eps_zero.xyy),
		SDF(pos + eps_zero.yxy),
		SDF(pos + eps_zero.yyx)) - c);
}

float castRay(vec3 rayOrigin, vec3 rayDir)
{
	float t = 0.0; // Stores current distance along ray
	
	for (int i = 0; i < 64; i++)
	{
		float res = SDF(rayOrigin + rayDir * t);
		if (res < (0.0001*t))
		{
			return t;
		}
		t += res;
	}
	
	return -1.0;
}

vec3 render(vec3 rayOrigin, vec3 rayDir)
{
	vec3 col;
	float t = castRay(rayOrigin, rayDir);

	vec3 L = normalize(vec3(sin(iTime)*1.0, cos(iTime*0.5)+0.5, -0.5));

	if (t == -1.0)
	{
		col = vec3(0.30, 0.36, 0.60) - rayDir.y*0.4;
	}
	else
	{   
		vec3 pos = rayOrigin + rayDir * t;
		vec3 N = calcNormal(pos);

		vec3 objectSurfaceColour = vec3(0.4, 0.8, 0.1);
		// L is vector from surface point to light, N is surface normal. N and L must be normalized!
		float NoL = max(dot(N, L), 0.0);
		vec3 LDirectional = vec3(1.80,1.27,0.99) * NoL;
		vec3 LAmbient = vec3(0.03, 0.04, 0.1);
		vec3 diffuse = objectSurfaceColour * (LDirectional + LAmbient);
		
		col = diffuse;
		
		
		float shadow = 0.0;
		vec3 shadowRayOrigin = pos + N * 0.01;
		vec3 shadowRayDir = L;
		float t = castRay(shadowRayOrigin, shadowRayDir);
		if (t >= -1.0)
		{
			shadow = 1.0;
		}
		col = mix(col, col*0.8, shadow);
		
		// Visualize normals:
		//col = N * vec3(0.5) + vec3(0.5);
	}
	
	return col;
}

vec3 getCameraRayDir(vec2 uv, vec3 camPos, vec3 camTarget)
{
	vec3 camForward = normalize(camTarget - camPos);
	vec3 camRight = normalize(cross(vec3(0.0, 1.0, 0.0), camForward));
	vec3 camUp = normalize(cross(camForward, camRight));

	// fPersp controls the camera's field of view. Try changing it!
	float fPersp = 2.0;
	vec3 vDir = normalize(uv.x * camRight + uv.y * camUp + camForward * fPersp);

	return vDir;
}

vec2 normalizeScreenCoords(vec2 screenCoord)
{
	vec2 result = 2.0 * (screenCoord/iResolution.xy - 0.5);
	result.x *= iResolution.x/iResolution.y; // Correct for aspect ratio
	result.y *= -1.0;
	return result;
}

void main()
{
	vec3 camPos = vec3(0, 0, -1);
	vec3 at = vec3(0, 0, 0);
	
	vec2 uv = normalizeScreenCoords(gl_FragCoord.xy);
	vec3 rayDir = getCameraRayDir(uv, camPos, at);  
	
	vec3 col = render(camPos, rayDir);
	
	fragColor = vec4(col,1.0); // Output to screen
}
