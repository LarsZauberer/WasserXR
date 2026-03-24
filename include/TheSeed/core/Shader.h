#ifndef TS_SHADER_H
#define TS_SHADER_H

#include <cglm/cglm.h>

/**
 * @brief Opaque shader structure
 */
typedef struct TS_Shader TS_Shader;

/**
 * @brief Creates a shader object on the heap
 * @param path Base path to the shader files (without .vert/.frag extension)
 * @return Pointer to the newly created shader object
 */
TS_Shader *ts_create_shader(const char *path);

/**
 * @brief Loads shader source code from filesystem
 * @param shader The shader object
 * @return 0 on success, 1 on failure
 */
int ts_load_shader(TS_Shader *shader);

/**
 * @brief Compiles the shader program
 * @param shader The shader object
 * @return 0 on success, 1 on failure
 */
int ts_compile_shader(TS_Shader *shader);

/**
 * @brief Activates the shader for use with OpenGL
 * @param shader The shader object
 * @return 0 on success, 1 on failure
 */
int ts_use_shader(TS_Shader *shader);

/**
 * @brief Destroys the shader and frees all resources
 * @param shader The shader object
 */
void ts_destroy_shader(TS_Shader *shader);

/**
 * @brief Set a float uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The float value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_1f(TS_Shader *shader, const char *name, float value);

/**
 * @brief Set an integer uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The integer value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_1i(TS_Shader *shader, const char *name, int value);

/**
 * @brief Set a vec2 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The vec2 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_2f(TS_Shader *shader, const char *name, const vec2 value);

/**
 * @brief Set a vec3 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The vec3 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_3f(TS_Shader *shader, const char *name, const vec3 value);

/**
 * @brief Set a vec4 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The vec4 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_4f(TS_Shader *shader, const char *name, const vec4 value);

/**
 * @brief Set a mat2 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The mat2 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_mat2(TS_Shader *shader, const char *name, const mat2 value);

/**
 * @brief Set a mat3 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The mat3 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_mat3(TS_Shader *shader, const char *name, const mat3 value);

/**
 * @brief Set a mat4 uniform in the shader
 * @param shader The shader object
 * @param name The uniform name
 * @param value The mat4 value
 * @return 0 on success, 1 on failure
 */
int ts_set_shader_uniform_mat4(TS_Shader *shader, const char *name, const mat4 value);

#endif
