#include <glad/gl.h>

#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/logging.h"
#include <assimp/cimport.h>
#include <assimp/postprocess.h>
#include <assimp/scene.h>
#include <glib.h>
#include <stdlib.h>

TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data) {
  TS_Mesh *mesh = (TS_Mesh *)malloc(sizeof(TS_Mesh));
  ts_assert(mesh, "Malloc returned NULL during ts_create_mesh_from_data");

  mesh->numIndices = (int)mesh_data->faces_size * 3;

  // Generate the buffers
  glGenVertexArrays(1, &mesh->vao);
  ts_assert(mesh->vao, "Vertex Array couldn't be allocated");
  glGenBuffers(1, &mesh->vbo);
  ts_assert(mesh->vbo, "Vertex Buffer couldn't be allocated");
  glGenBuffers(1, &mesh->ebo);
  ts_assert(mesh->ebo, "Element Buffer couldn't be allocated");

  // // Bind the buffers
  glBindVertexArray(mesh->vao);

  // Move vertices over
  glBindBuffer(GL_ARRAY_BUFFER, mesh->vbo);
  glBufferData(GL_ARRAY_BUFFER,
               (long)sizeof(float) * 3 * (long)mesh_data->vertices_size,
               mesh_data->vertices, GL_STATIC_DRAW);

  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, mesh->ebo);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER,
               (long)sizeof(unsigned int) * 3 * mesh_data->faces_size,
               mesh_data->indices, GL_STATIC_DRAW);

  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void *)0);
  glEnableVertexAttribArray(0);

  // Unbind
  glBindBuffer(GL_ARRAY_BUFFER, 0);
  // You are not allowed to unbind the ebo because it is stored in the vao
  // directly
  glBindVertexArray(0);
  return mesh;
}

void ts_destroy_mesh(TS_Mesh *mesh) {
  glDeleteVertexArrays(1, &mesh->vao);
  glDeleteBuffers(1, &mesh->vbo);
  glDeleteBuffers(1, &mesh->ebo);
  free(mesh);
}
