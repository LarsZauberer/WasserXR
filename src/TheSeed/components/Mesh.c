#include "TheSeed/components/Mesh.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/ecs/Scene.h"
#include <glad/gl.h>
#include <stdlib.h>

void *ts_create_TS_Model() {
  TS_Mesh *mesh = (TS_Mesh *)malloc(sizeof(TS_Mesh));

  mesh->vertices = (float *)malloc(sizeof(float) * 3 * 4);
  mesh->indices = (unsigned int *)malloc(sizeof(unsigned int) * 3 * 2);

  // Model data
  // TODO: When implementing mesh loading have a mesh cache (Return the right
  // VBO)
  mesh->vertices[0] = 0.5f;
  mesh->vertices[1] = 0.6f;
  mesh->vertices[2] = 0.0f;

  mesh->vertices[3] = 0.5f;
  mesh->vertices[4] = -0.5f;
  mesh->vertices[5] = 0.0f;

  mesh->vertices[6] = -0.5f;
  mesh->vertices[7] = -0.5f;
  mesh->vertices[8] = 0.0f;

  mesh->vertices[9] = -0.5f;
  mesh->vertices[10] = 0.5f;
  mesh->vertices[11] = 0.0f;

  mesh->indices[0] = 0;
  mesh->indices[1] = 1;
  mesh->indices[2] = 3;

  mesh->indices[3] = 1;
  mesh->indices[4] = 2;
  mesh->indices[5] = 3;

  // Add shader
  // TODO: Add shader cache
  mesh->shader = ts_create_shader("shaders/base");
  ts_load_shader(mesh->shader);
  ts_compile_shader(mesh->shader);

  // Generating all the vertex arrays and buffers
  glGenVertexArrays(1, &mesh->vao);
  glGenBuffers(1, &mesh->vbo);
  glGenBuffers(1, &mesh->ebo);

  // Load the model
  glBindVertexArray(mesh->vao);

  glBindBuffer(GL_ARRAY_BUFFER, mesh->vbo);
  glBufferData(GL_ARRAY_BUFFER, sizeof(float) * 3 * 4, mesh->vertices,
               GL_STATIC_DRAW);

  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, mesh->ebo);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(unsigned int) * 3 * 2,
               mesh->indices, GL_STATIC_DRAW);

  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void *)0);
  glEnableVertexAttribArray(0);

  // Unbind
  glBindBuffer(GL_ARRAY_BUFFER, 0);
  // You are not allowed to unbind the ebo because it is stored in the vao
  // directly
  glBindVertexArray(0);

  return mesh;
}

void ts_destroy_TS_Mesh(void *p) {
  TS_Mesh *mesh = (TS_Mesh *)p;
  free(mesh->vertices);
  free(mesh->indices);
  ts_destroy_shader(mesh->shader);
  glDeleteVertexArrays(1, &mesh->vao);
  glDeleteBuffers(1, &mesh->vbo);
  glDeleteBuffers(1, &mesh->ebo);
  free(mesh);
  return;
}
