#include "TheSeed/core/Shader.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include <cglm/cglm.h>
#include <glad/gl.h>
#include <glib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct TS_Shader {
  char *path;
  unsigned int vertex_shader;
  unsigned int fragment_shader;
  unsigned int program;
  int is_loaded;
  int is_compiled;
  char *vertex_source;
  char *fragment_source;
};

TS_Shader *ts_create_shader(const char *path) {
  TS_Shader *shader = (TS_Shader *)malloc(sizeof(TS_Shader));
  ts_assert(shader, "Malloc returned NULL in ts_create_shader");

  shader->path = ts_copy_char_ptr(path);
  shader->vertex_source = NULL;
  shader->fragment_source = NULL;
  shader->program = 0;
  shader->is_loaded = 0;
  shader->is_compiled = 0;

  return shader;
}

int ts_load_shader(TS_Shader *shader) {
  ts_assert_abort_value(
      shader, 1,
      "Shader is NULL during ts_load_shader. Call `ts_create_shader` first");

  // Build vertex shader path
  GString *vertex_path_gstring = g_string_new(shader->path);
  g_string_append(vertex_path_gstring, ".vert");
  char *vertex_path = g_string_free(vertex_path_gstring, FALSE);

  // Build fragment shader path
  GString *fragment_path_gstring = g_string_new(shader->path);
  g_string_append(fragment_path_gstring, ".frag");
  char *fragment_path = g_string_free(fragment_path_gstring, FALSE);

  // Load vertex shader
  shader->vertex_source = ts_read_file(vertex_path);
  if (shader->vertex_source == NULL) {
    free(vertex_path);
    free(fragment_path);
    return 1;
  }

  // Load fragment shader
  shader->fragment_source = ts_read_file(fragment_path);
  if (shader->fragment_source == NULL) {
    free(vertex_path);
    free(fragment_path);
    free(shader->vertex_source);
    shader->vertex_source = NULL;
    return 1;
  }

  free(vertex_path);
  free(fragment_path);

  shader->is_loaded = 1;
  return 0;
}

int ts_compile_shader(TS_Shader *shader) {
  ts_assert_abort_value(shader, 1, "Shader is NULL during ts_compile_shader");

  ts_assert_abort_value(
      shader->is_loaded, 1,
      "Error: Shader not loaded. Call ts_load_shader() first");

  int success;
  char info_log[512];

  unsigned int vertex_shader = 0;
  unsigned int fragment_shader = 0;

  // Compile vertex shader
  vertex_shader = glCreateShader(GL_VERTEX_SHADER);
  const char *vertex_src = shader->vertex_source;
  glShaderSource(vertex_shader, 1, &vertex_src, NULL);
  glCompileShader(vertex_shader);

  glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(vertex_shader, 512, NULL, info_log);
    ts_error("Error: Vertex shader compilation failed\n%s", info_log);
    free(shader->vertex_source);
    free(shader->fragment_source);
    shader->is_loaded = 0;
    return 1;
  }

  // Compile fragment shader
  fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
  const char *fragment_src = shader->fragment_source;
  glShaderSource(fragment_shader, 1, &fragment_src, NULL);
  glCompileShader(fragment_shader);

  glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(fragment_shader, 512, NULL, info_log);
    ts_error("Error: Fragment shader compilation failed\n%s", info_log);
    free(shader->vertex_source);
    free(shader->fragment_source);
    glDeleteShader(vertex_shader);
    shader->is_loaded = 0;
    return 1;
  }

  // Link shader program
  shader->program = glCreateProgram();
  glAttachShader(shader->program, vertex_shader);
  glAttachShader(shader->program, fragment_shader);
  glLinkProgram(shader->program);

  glGetProgramiv(shader->program, GL_LINK_STATUS, &success);
  if (!success) {
    glGetProgramInfoLog(shader->program, 512, NULL, info_log);
    ts_error("Error: Shader program linking failed\n%s", info_log);
    free(shader->vertex_source);
    free(shader->fragment_source);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);
    glDeleteProgram(shader->program);
    shader->is_loaded = 0;
    return 1;
  }

  // Clean up everything unneeded
  free(shader->vertex_source);
  free(shader->fragment_source);
  glDeleteShader(vertex_shader);
  glDeleteShader(fragment_shader);

  shader->is_loaded = 0;
  shader->is_compiled = 1;
  return 0;
}

int ts_use_shader(TS_Shader *shader) {
  ts_assert_abort_value(shader, -1, "Shader is NULL during ts_use_shader");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled. Call ts_compile_shader() first "
             "before trying to use the shader");
    return 1;
  }

  glUseProgram(shader->program);
  return 0;
}

void ts_destroy_shader(TS_Shader *shader) {
  if (!shader) {
    return;
  }

  if (shader->is_compiled) {
    glDeleteProgram(shader->program);
  }

  if (shader->path) {
    free(shader->path);
  }

  free(shader);
}

int ts_set_shader_uniform_1f(TS_Shader *shader, const char *name, float value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_1f");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniform1f(location, value);
  return 0;
}

int ts_set_shader_uniform_1i(TS_Shader *shader, const char *name, int value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_1i");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniform1i(location, value);
  return 0;
}

int ts_set_shader_uniform_2f(TS_Shader *shader, const char *name, const vec2 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_2f");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniform2f(location, value[0], value[1]);
  return 0;
}

int ts_set_shader_uniform_3f(TS_Shader *shader, const char *name, const vec3 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_3f");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniform3f(location, value[0], value[1], value[2]);
  return 0;
}

int ts_set_shader_uniform_4f(TS_Shader *shader, const char *name, const vec4 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_4f");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniform4f(location, value[0], value[1], value[2], value[3]);
  return 0;
}

int ts_set_shader_uniform_mat2(TS_Shader *shader, const char *name,
                               const mat2 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_mat2");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniformMatrix2fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}

int ts_set_shader_uniform_mat3(TS_Shader *shader, const char *name,
                               const mat3 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_mat3");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniformMatrix3fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}

int ts_set_shader_uniform_mat4(TS_Shader *shader, const char *name,
                               const mat4 value) {
  ts_assert_abort_value(shader, -1,
                        "Shader is NULL during ts_set_shader_uniform_mat4");

  if (!shader->is_compiled) {
    ts_error("Error: Shader not compiled");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    ts_warn("Warning: Uniform '%s' not found in shader", name);
    return 1;
  }

  glUniformMatrix4fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}
