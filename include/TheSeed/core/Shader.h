#ifndef TS_SHADER_H
#define TS_SHADER_H

/**
 * @brief Opaque shader structure
 */
typedef struct TS_Shader TS_Shader;

/**
 * @brief Creates a shader object on the heap
 * @param path Base path to the shader files (without .vert/.frag extension)
 * @return Pointer to the newly created shader object
 */
TS_Shader *ts_create_shader(char *path);

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

#endif
