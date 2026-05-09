#ifndef TS_MESH_INTERNAL_H
#define TS_MESH_INTERNAL_H

#include <stddef.h>

/**
 * @brief Mesh data structure for storing raw mesh data from files
 */
typedef struct TS_Mesh_Data TS_Mesh_Data;

/**
 * @brief GPU mesh structure for OpenGL rendering
 */
typedef struct TS_Mesh TS_Mesh;

/**
 * @brief Internal mesh data structure definition
 */
struct TS_Mesh_Data {
  unsigned int vertices_size;
  unsigned int faces_size;

  float *vertices;
  float *normals;
  float *uvs;
  unsigned int *indices;
};

/**
 * @brief Internal mesh structure definition
 */
struct TS_Mesh {
  int numIndices;
  unsigned int vao;
  unsigned int vertexVbo;
  unsigned int normalVbo;
  unsigned int uvVbo;
  unsigned int ebo;
};

/**
 * @brief Reads mesh data from a file
 * @param n Output parameter for the number of meshes read
 * @param filename Path to the mesh file
 * @return Array of mesh data structures
 */
TS_Mesh_Data *ts_read_mesh_data(unsigned int *n, const char *filename);

/**
 * @brief Destroys mesh data and frees all resources
 * @param mesh The mesh data to destroy
 */
void ts_destroy_mesh_data(TS_Mesh_Data *mesh);

/**
 * @brief Creates a GPU mesh from mesh data
 * @param mesh_data The mesh data to upload to GPU
 * @return Pointer to the newly created mesh object
 */
TS_Mesh *ts_create_mesh_from_data(TS_Mesh_Data *mesh_data);

/**
 * @brief Destroys a GPU mesh and frees all resources
 * @param mesh The mesh to destroy
 */
void ts_destroy_mesh(TS_Mesh *mesh);

#endif
