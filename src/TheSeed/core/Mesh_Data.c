#include "TheSeed/core/Mesh_Data.h"
#include "TheSeed/core/logging.h"
#include "assimp/cimport.h"
#include "assimp/mesh.h"
#include "assimp/postprocess.h"
#include "assimp/scene.h"
#include "glib.h"
#include <stdlib.h>

typedef struct aiScene aiScene;
typedef struct aiNode aiNode;
typedef struct aiMesh aiMesh;
typedef struct aiFace aiFace;

// Recursive function to handle all the model nodes
static TS_Mesh_Data ts_process_mesh(const aiMesh *mesh) {
  TS_Mesh_Data mesh_data;

  float *vertices = malloc(sizeof(float) * mesh->mNumVertices * 3);
  float *normals = malloc(sizeof(float) * mesh->mNumVertices * 3);
  float *uvs = malloc(sizeof(float) * mesh->mNumVertices * 2);
  ts_assert(
      vertices,
      "Malloc returned null for the vertices creation during ts_process_mesh");
  ts_assert(
      normals,
      "Malloc returned null for the normals creation during ts_process_mesh");
  ts_assert(uvs,
            "Malloc returned null for the UVs creation during ts_process_mesh");
  for (unsigned int i = 0; i < mesh->mNumVertices; i++) {
    vertices[(i * 3) + 0] = mesh->mVertices[i].x;
    vertices[(i * 3) + 1] = mesh->mVertices[i].y;
    vertices[(i * 3) + 2] = mesh->mVertices[i].z;

    if (mesh->mNormals) {
      normals[(i * 3) + 0] = mesh->mNormals[i].x;
      normals[(i * 3) + 1] = mesh->mNormals[i].y;
      normals[(i * 3) + 2] = mesh->mNormals[i].z;
    } else {
      normals[(i * 3) + 0] = 0.0F;
      normals[(i * 3) + 1] = 0.0F;
      normals[(i * 3) + 2] = 1.0F;
    }

    if (mesh->mTextureCoords[0]) {
      uvs[(i * 2) + 0] = mesh->mTextureCoords[0][i].x;
      uvs[(i * 2) + 1] = mesh->mTextureCoords[0][i].y;
    } else {
      uvs[(i * 2) + 0] = 0.0F;
      uvs[(i * 2) + 1] = 0.0F;
    }
  }

  mesh_data.vertices_size = mesh->mNumVertices;
  mesh_data.vertices = vertices;
  mesh_data.normals = normals;
  mesh_data.uvs = uvs;

  unsigned int *indices = malloc(sizeof(unsigned int) * mesh->mNumFaces * 3);
  ts_assert(
      indices,
      "Malloc returned null for indicies creation during ts_process_mesh");

  for (unsigned int i = 0; i < mesh->mNumFaces; i++) {
    const aiFace face = mesh->mFaces[i];
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
                            const aiNode *node) {
  // process all the node's meshes (if any)
  for (unsigned int i = 0; i < node->mNumMeshes; i++) {
    const aiMesh *mesh = scene->mMeshes[node->mMeshes[i]];
    const TS_Mesh_Data new_mesh = ts_process_mesh(mesh);
    g_array_append_val(mesh_data, new_mesh);
  }
  // then do the same for each of its children
  for (unsigned int i = 0; i < node->mNumChildren; i++) {
    ts_process_node(mesh_data, scene, node->mChildren[i]);
  }
}

TS_Mesh_Data *ts_read_mesh_data(unsigned int *n, const char *filename) {
  const aiScene *scene =
      aiImportFile(filename, aiProcess_Triangulate | aiProcess_FlipUVs |
                                 aiProcess_GenSmoothNormals);

  if (!scene) {
    ts_error("Failed to load the model file %s: %s", filename,
             aiGetErrorString());
    *n = 0;
    return NULL;
  }

  GArray *output_meshes = g_array_new(FALSE, FALSE, sizeof(TS_Mesh_Data));
  ts_process_node(output_meshes, scene, scene->mRootNode);

  aiReleaseImport(scene);

  *n = output_meshes->len;
  return (TS_Mesh_Data *)g_array_free(output_meshes, FALSE);
}

void ts_destroy_mesh_data(TS_Mesh_Data *mesh) {
  free(mesh->indices);
  free(mesh->normals);
  free(mesh->uvs);
  free(mesh->vertices);
}
