#include <glad/gl.h>

#include "TheSeed/core/Mesh.h"
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
  for (unsigned int i = 0; i < mesh->mNumVertices; i++) {
    vertices[i * 3 + 0] = mesh->mVertices[i].x;
    vertices[i * 3 + 1] = mesh->mVertices[i].y;
    vertices[i * 3 + 2] = mesh->mVertices[i].z;

    normals[i * 3 + 0] = mesh->mNormals[i].x;
    normals[i * 3 + 1] = mesh->mNormals[i].y;
    normals[i * 3 + 2] = mesh->mNormals[i].z;
  }

  mesh_data.vertices_size = mesh->mNumVertices;
  mesh_data.vertices = vertices;
  mesh_data.normals = normals;

  GArray *indices = g_array_new(FALSE, FALSE, sizeof(unsigned int));

  for (unsigned int i = 0; i < mesh->mNumFaces; i++) {
    aiFace face = mesh->mFaces[i];
    g_assert(face.mNumIndices == 3);
    for (unsigned int j = 0; j < face.mNumIndices; j++) {
      g_array_append_val(indices, face.mIndices[j]);
    }
  }

  mesh_data.faces_size = mesh->mNumFaces;
  mesh_data.indices = (unsigned int *)g_array_free(indices, FALSE);

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

  return;
}

int ts_read_mesh_data(TS_Mesh_Data *out, char *filename) {
  const aiScene *scene =
      aiImportFile(filename, aiProcess_Triangulate | aiProcess_FlipUVs);

  if (!scene) {
    printf("Failed to load the model file %s: %s\n", filename,
           aiGetErrorString());
  }

  GArray *output_meshes = g_array_new(FALSE, FALSE, sizeof(TS_Mesh_Data));
  ts_process_node(output_meshes, scene, scene->mRootNode);

  unsigned int n = output_meshes->len;
  out = (TS_Mesh_Data *)g_array_free(output_meshes, FALSE);
  return n;
}

void ts_destroy_mesh_data(TS_Mesh_Data *mesh) {
  free(mesh->indices);
  free(mesh->normals);
  free(mesh->vertices);
  return;
}

TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data) {}

void ts_destroy_mesh(TS_Mesh *mesh) {}
