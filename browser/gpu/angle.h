/* The EGL and OpenGL ES headers gl.aye reads with @cimport (eglext.h
 * needs egl.h first). include/ is a copy of the headers of the ANGLE
 * revision in angle.rev (the archive's include/, as build_angle.sh
 * packages it), so Dweller builds without ANGLE; copy them again when
 * angle.rev changes. */
#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <GLES3/gl3.h>
#include <GLES2/gl2ext.h>
