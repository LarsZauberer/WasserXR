#include "TheSeed/components/Camera.h"
#include "TheSeed/core/Mesh.h"
#include "glad/gl.h"

#include "GLFW/glfw3.h"
#include "TheSeed/components/Model.h"
#include "TheSeed/components/Transform.h"
#include "TheSeed/components/Window.h"
#include "TheSeed/core/Shader.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/ecs/Scene.h"
#include "cglm/affine-pre.h"
#include "cglm/affine.h"
#include "cglm/cam.h"
#include "cglm/mat4.h"
#include "cglm/types.h"
#include "cglm/util.h"
#include "cglm/vec4.h"
#include <stdio.h>

TS_System_Groups ts_groups_ts_mesh_renderer = 3;

TS_System_Groups ts_select_ts_mesh_renderer(TS_Scene *scene,
                                            const TS_Entity entity) {
  size_t normal_object =
      ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Model");
  size_t camera_object =
      ts_entity_get_component(scene, entity, "TS_Transform") &&
      ts_entity_get_component(scene, entity, "TS_Camera");
  size_t window = (size_t)ts_entity_get_component(scene, entity, "TS_Window");
  if (window) {
    return 1;
  }
  if (camera_object) {
    return 2;
  }
  if (normal_object) {
    return 3;
  }
  return 0;
}

void ts_system_ts_mesh_renderer(TS_Scene *scene, TS_Entity **entities,
                                const size_t *sizes) {
  TS_Entity camera_entity;
  TS_Camera *camera;
  TS_Transform *cam_transform;

  TS_Entity window_entity;
  TS_Window *window;
  GLFWwindow *glfw_window;

  if (!sizes[0]) {
    ts_warn("No window!\n");
    return;
  }

  if (!sizes[1]) {
    ts_warn("No camera!\n");
    return;
  }

  window_entity = entities[0][0];
  window =
      (TS_Window *)ts_entity_get_component(scene, window_entity, "TS_Window");
  glfw_window = ts_get(scene, window, "window");
  if (!glfw_window) {
    ts_warn("Window field is NULL");
    return;
  }

  camera_entity = entities[1][0];
  cam_transform = (TS_Transform *)ts_entity_get_component(scene, camera_entity,
                                                          "TS_Transform");
  camera =
      (TS_Camera *)ts_entity_get_component(scene, camera_entity, "TS_Camera");

  for (size_t i = 0; i < sizes[2]; i++) {
    // Normal mesh entity
    TS_Entity entity = entities[2][i];

    TS_Model *model = ts_entity_get_component(scene, entity, "TS_Model");
    TS_Transform *transform =
        ts_entity_get_component(scene, entity, "TS_Transform");

    TS_Mesh **meshes = ts_get(scene, model, "meshes");
    unsigned int num_meshes =
        *(unsigned int *)ts_get(scene, model, "num_meshes");
    TS_Shader *shader = ts_get(scene, model, "shader");

    // Check if the model is loaded yet
    if (meshes == NULL || shader == NULL) {
      ts_warn("Model is not properly loaded.");
      continue;
    }

    mat4 model_transform;
    mat4 view_transform;
    mat4 projection_transform;
    glm_mat4_identity(model_transform);
    glm_mat4_identity(view_transform);
    glm_mat4_identity(projection_transform);

    // Create the transformation matrix

    // World Space placement
    vec3 position = {*(float *)ts_get(scene, transform, "x"),
                     *(float *)ts_get(scene, transform, "y"),
                     *(float *)ts_get(scene, transform, "z")};
    vec3 scale = {*(float *)ts_get(scene, transform, "sx"),
                  *(float *)ts_get(scene, transform, "sy"),
                  *(float *)ts_get(scene, transform, "sz")};
    glm_translate(model_transform, position);
    glm_rotate_x(model_transform,
                 glm_rad(*(float *)ts_get(scene, transform, "rx")),
                 model_transform);
    glm_rotate_y(model_transform,
                 glm_rad(*(float *)ts_get(scene, transform, "ry")),
                 model_transform);
    glm_rotate_z(model_transform,
                 glm_rad(*(float *)ts_get(scene, transform, "rz")),
                 model_transform);
    glm_scale(model_transform, scale);

    // Camera placement
    vec3 camera_position = {*(float *)ts_get(scene, cam_transform, "x"),
                            *(float *)ts_get(scene, cam_transform, "y"),
                            *(float *)ts_get(scene, cam_transform, "z")};
    vec4 camera_pos_4;
    glm_vec4(camera_position, 1.0F, camera_pos_4);
    glm_vec4_negate(camera_pos_4);
    glm_translate(view_transform, camera_pos_4);
    glm_rotate_x(view_transform,
                 glm_rad(*(float *)ts_get(scene, cam_transform, "rx")),
                 view_transform);
    glm_rotate_y(view_transform,
                 glm_rad(*(float *)ts_get(scene, cam_transform, "ry")),
                 view_transform);
    glm_rotate_z(view_transform,
                 glm_rad(*(float *)ts_get(scene, cam_transform, "rz")),
                 view_transform);

    // Perspective
    int width;
    int height;
    float fov = *(float *)ts_get(scene, camera, "fov");
    float near = *(float *)ts_get(scene, camera, "near");
    float far = *(float *)ts_get(scene, camera, "far");
    glfwGetWindowSize(glfw_window, &width, &height);
    glm_perspective(glm_rad(fov), (float)width / (float)height, near, far,
                    projection_transform);

    int status = ts_use_shader(shader);
    if (status) {
      ts_warn("Shader couldn't be applied to the mesh of entity %ld. Skipping "
              "rendering of entity",
              entity);
      continue;
    }

    // Put everything to the respective uniforms in the shader
    ts_set_shader_uniform_mat4(shader, "model", model_transform);
    ts_set_shader_uniform_mat4(shader, "view", view_transform);
    ts_set_shader_uniform_mat4(shader, "projection", projection_transform);

    // Draw the meshes
    for (unsigned int i = 0; i < num_meshes; i++) {
      glBindVertexArray(meshes[i]->vao);

      glDrawElements(GL_TRIANGLES, meshes[i]->numIndices, GL_UNSIGNED_INT, 0);
    }
  }
}
