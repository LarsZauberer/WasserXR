#include "TheSeed/core/Shader.h"
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
  GString *vertex_source;
  GString *fragment_source;
};

TS_Shader *ts_create_shader(char *path) {
  TS_Shader *shader = (TS_Shader *)malloc(sizeof(TS_Shader));

  shader->path = ts_copy_char_ptr(path);
  shader->vertex_source = NULL;
  shader->fragment_source = NULL;
  shader->program = 0;
  shader->is_loaded = 0;
  shader->is_compiled = 0;

  return shader;
}

int ts_load_shader(TS_Shader *shader) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  // Build vertex shader path
  size_t path_len = strlen(shader->path);
  char *vertex_path =
      (char *)malloc(path_len + 6); // +5 for ".vert" +1 for null
  strcpy(vertex_path, shader->path);
  strcat(vertex_path, ".vert");

  // Build fragment shader path
  char *fragment_path =
      (char *)malloc(path_len + 6); // +5 for ".frag" +1 for null
  strcpy(fragment_path, shader->path);
  strcat(fragment_path, ".frag");

  // Load vertex shader
  if (ts_read_file_to_gstring(vertex_path, &shader->vertex_source) != 0) {
    free(vertex_path);
    free(fragment_path);
    return 1;
  }

  // Load fragment shader
  if (ts_read_file_to_gstring(fragment_path, &shader->fragment_source) != 0) {
    free(vertex_path);
    free(fragment_path);
    g_string_free(shader->vertex_source, TRUE);
    shader->vertex_source = NULL;
    return 1;
  }

  free(vertex_path);
  free(fragment_path);

  shader->is_loaded = 1;
  return 0;
}

int ts_compile_shader(TS_Shader *shader) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_loaded) {
    fprintf(stderr, "Error: Shader not loaded. Call ts_load_shader() first\n");
    return 1;
  }

  int success;
  char info_log[512];

  unsigned int vertex_shader = 0;
  unsigned int fragment_shader = 0;

  // Compile vertex shader
  vertex_shader = glCreateShader(GL_VERTEX_SHADER);
  const char *vertex_src = shader->vertex_source->str;
  glShaderSource(vertex_shader, 1, &vertex_src, NULL);
  glCompileShader(vertex_shader);

  glGetShaderiv(vertex_shader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(vertex_shader, 512, NULL, info_log);
    fprintf(stderr, "Error: Vertex shader compilation failed\n%s\n", info_log);
    g_string_free(shader->vertex_source, TRUE);
    g_string_free(shader->fragment_source, TRUE);
    shader->is_loaded = 0;
    return 1;
  }

  // Compile fragment shader
  fragment_shader = glCreateShader(GL_FRAGMENT_SHADER);
  const char *fragment_src = shader->fragment_source->str;
  glShaderSource(fragment_shader, 1, &fragment_src, NULL);
  glCompileShader(fragment_shader);

  glGetShaderiv(fragment_shader, GL_COMPILE_STATUS, &success);
  if (!success) {
    glGetShaderInfoLog(fragment_shader, 512, NULL, info_log);
    fprintf(stderr, "Error: Fragment shader compilation failed\n%s\n",
            info_log);
    g_string_free(shader->vertex_source, TRUE);
    g_string_free(shader->fragment_source, TRUE);
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
    fprintf(stderr, "Error: Shader program linking failed\n%s\n", info_log);
    g_string_free(shader->vertex_source, TRUE);
    g_string_free(shader->fragment_source, TRUE);
    glDeleteShader(vertex_shader);
    glDeleteShader(fragment_shader);
    glDeleteProgram(shader->program);
    shader->is_loaded = 0;
    return 1;
  }

  // Clean up everything unneeded
  g_string_free(shader->vertex_source, TRUE);
  g_string_free(shader->fragment_source, TRUE);
  glDeleteShader(vertex_shader);
  glDeleteShader(fragment_shader);

  shader->is_loaded = 0;
  shader->is_compiled = 1;
  return 0;
}

int ts_use_shader(TS_Shader *shader) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr,
            "Error: Shader not compiled. Call ts_compile_shader() first\n");
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

int ts_set_shader_uniform_1f(TS_Shader *shader, char *name, float value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniform1f(location, value);
  return 0;
}

int ts_set_shader_uniform_1i(TS_Shader *shader, char *name, int value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniform1i(location, value);
  return 0;
}

int ts_set_shader_uniform_2f(TS_Shader *shader, char *name, const vec2 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniform2f(location, value[0], value[1]);
  return 0;
}

int ts_set_shader_uniform_3f(TS_Shader *shader, char *name, const vec3 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniform3f(location, value[0], value[1], value[2]);
  return 0;
}

int ts_set_shader_uniform_4f(TS_Shader *shader, char *name, const vec4 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniform4f(location, value[0], value[1], value[2], value[3]);
  return 0;
}

int ts_set_shader_uniform_mat2(TS_Shader *shader, char *name,
                               const mat2 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniformMatrix2fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}

int ts_set_shader_uniform_mat3(TS_Shader *shader, char *name,
                               const mat3 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniformMatrix3fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}

int ts_set_shader_uniform_mat4(TS_Shader *shader, char *name,
                               const mat4 value) {
  if (!shader) {
    fprintf(stderr, "Error: NULL shader pointer\n");
    return 1;
  }

  if (!shader->is_compiled) {
    fprintf(stderr, "Error: Shader not compiled\n");
    return 1;
  }

  GLint location = glGetUniformLocation(shader->program, name);
  if (location == -1) {
    fprintf(stderr, "Warning: Uniform '%s' not found in shader\n", name);
    return 1;
  }

  glUniformMatrix4fv(location, 1, GL_FALSE, (float *)value);
  return 0;
}
