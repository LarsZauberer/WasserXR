#include <glad/gl.h>

#include "TheSeed/core/Mesh.h"
#include "TheSeed/core/logging.h"
#include "assimp/mesh.h"
#include <assimp/cimport.h>
#include <assimp/postprocess.h>
#include <assimp/scene.h>
#include <glib.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct aiScene aiScene;
typedef struct aiNode aiNode;
typedef struct aiMesh aiMesh;
typedef struct aiFace aiFace;

// Recursive function to handle all the model nodes
static TS_Mesh_Data ts_process_mesh(aiMesh *mesh) {
  TS_Mesh_Data mesh_data;

  float *vertices = malloc(sizeof(float) * mesh->mNumVertices * 3);
  float *normals = malloc(sizeof(float) * mesh->mNumVertices * 3);
  ts_assert(
      vertices,
      "Malloc returned null for the vertices creation during ts_process_mesh");
  ts_assert(
      normals,
      "Malloc returned null for the normals creation during ts_process_mesh");
  for (unsigned int i = 0; i < mesh->mNumVertices; i++) {
    vertices[(i * 3) + 0] = mesh->mVertices[i].x;
    vertices[(i * 3) + 1] = mesh->mVertices[i].y;
    vertices[(i * 3) + 2] = mesh->mVertices[i].z;

    normals[(i * 3) + 0] = mesh->mNormals[i].x;
    normals[(i * 3) + 1] = mesh->mNormals[i].y;
    normals[(i * 3) + 2] = mesh->mNormals[i].z;
  }

  mesh_data.vertices_size = mesh->mNumVertices;
  mesh_data.vertices = vertices;
  mesh_data.normals = normals;

  unsigned int *indices = malloc(sizeof(unsigned int) * mesh->mNumFaces * 3);
  ts_assert(
      indices,
      "Malloc returned null for indicies creation during ts_process_mesh");

  for (unsigned int i = 0; i < mesh->mNumFaces; i++) {
    aiFace face = mesh->mFaces[i];
    ts_assert(face.mNumIndices == 3,
              "The mesh being processed is not a triangle mesh. Meshes other "
              "than triangle meshes are not supported");
    for (unsigned int j = 0; j < face.mNumIndices; j++) {
      indices[(i * 3) + j] = face.mIndices[j];
    }
  }

  mesh_data.faces_size = mesh->mNumFaces;
  mesh_data.indices = indices;
  return mesh_data;
}

static void ts_process_node(GArray *mesh_data, const aiScene *scene,
                            aiNode *node) {
  // process all the node's meshes (if any)
  for (unsigned int i = 0; i < node->mNumMeshes; i++) {
    aiMesh *mesh = scene->mMeshes[node->mMeshes[i]];
    TS_Mesh_Data new_mesh = ts_process_mesh(mesh);
    g_array_append_val(mesh_data, new_mesh);
  }
  // then do the same for each of its children
  for (unsigned int i = 0; i < node->mNumChildren; i++) {
    ts_process_node(mesh_data, scene, node->mChildren[i]);
  }
}

TS_Mesh_Data *ts_read_mesh_data(unsigned int *n, char *filename) {
  const aiScene *scene =
      aiImportFile(filename, aiProcess_Triangulate | aiProcess_FlipUVs);

  if (!scene) {
    ts_error("Failed to load the model file %s: %s", filename,
             aiGetErrorString());
    *n = 0;
    return NULL;
  }

  GArray *output_meshes = g_array_new(FALSE, FALSE, sizeof(TS_Mesh_Data));
  ts_process_node(output_meshes, scene, scene->mRootNode);

  *n = output_meshes->len;
  return (TS_Mesh_Data *)g_array_free(output_meshes, FALSE);
}

void ts_destroy_mesh_data(TS_Mesh_Data *mesh) {
  free(mesh->indices);
  free(mesh->normals);
  free(mesh->vertices);
}

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
