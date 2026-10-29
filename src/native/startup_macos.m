#import <AppKit/AppKit.h>
#import <ServiceManagement/ServiceManagement.h>
#include <stdio.h>

static BOOL CinnaStartupAvailable(void) {
    return [NSBundle.mainBundle.bundleURL.pathExtension isEqualToString:@"app"];
}

int Cinna_StartupStatus(void) {
    if (@available(macOS 13.0, *)) {
        if (!CinnaStartupAvailable()) return -1;
        return (int)SMAppService.mainAppService.status;
    }
    return -1;
}

bool Cinna_SetStartup(bool enabled, char *error, size_t capacity) {
    @autoreleasepool {
        if (@available(macOS 13.0, *)) {
            if (CinnaStartupAvailable()) {
                NSError *failure = nil;
                BOOL success = enabled
                    ? [SMAppService.mainAppService registerAndReturnError:&failure]
                    : [SMAppService.mainAppService unregisterAndReturnError:&failure];
                if (!success && enabled && SMAppService.mainAppService.status == SMAppServiceStatusRequiresApproval) success = YES;
                if (!success) snprintf(error, capacity, "%s", (failure.localizedDescription ?: @"Could not change the login item.").UTF8String);
                return success;
            }
        }
        snprintf(error, capacity, "%s", "Launch at login requires the bundled app and macOS 13 or later.");
        return false;
    }
}

void Cinna_OpenLoginSettings(void) {
    if (@available(macOS 13.0, *)) {
        [SMAppService openSystemSettingsLoginItems];
    }
}

const char *Cinna_StartupDescription(void) {
    static char description[1024];
    snprintf(description, sizeof(description), "bundle=%s; identifier=%s; service=%d", NSBundle.mainBundle.bundleURL.path.UTF8String, NSBundle.mainBundle.bundleIdentifier.UTF8String, Cinna_StartupStatus());
    return description;
}
