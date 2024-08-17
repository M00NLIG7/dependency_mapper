
package main

/*
#include <stdlib.h>

typedef struct {
    const char* source;
    const char* data;
} CollectedData;
*/
import "C"

//export CollectData
func CollectData() C.CollectedData {
    source := C.CString("go_plugin")
    
    data := C.CString("Data collected from Go plugin")
    
    return C.CollectedData{
        source: source,
        data:   data,
    }
}

func main() {}
