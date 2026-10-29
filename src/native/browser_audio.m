#import <Foundation/Foundation.h>
#import <CoreAudio/CoreAudio.h>

static BOOL Cinna_IsBrowser(NSString *bundle) {
    NSString *identifier = bundle.lowercaseString;
    for (NSString *prefix in @[@"com.google.chrome", @"com.apple.safari", @"com.apple.webkit", @"org.mozilla.firefox", @"org.mozilla.nightly", @"app.zen-browser", @"com.microsoft.edgemac", @"com.brave.browser", @"company.thebrowser.browser", @"com.operasoftware.opera", @"com.vivaldi.vivaldi"]) {
        if ([identifier isEqualToString:prefix] || [identifier hasPrefix:[prefix stringByAppendingString:@"."]]) return YES;
    }
    return NO;
}

int Cinna_BrowserAudioStatus(void) {
    @autoreleasepool {
        if ([NSProcessInfo.processInfo isOperatingSystemAtLeastVersion:(NSOperatingSystemVersion){14, 4, 0}]) {
            AudioObjectPropertyAddress list = { kAudioHardwarePropertyProcessObjectList, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain };
            UInt32 bytes = 0;
            if (AudioObjectGetPropertyDataSize(kAudioObjectSystemObject, &list, 0, NULL, &bytes) != noErr) return -2;
            if (bytes == 0) return 0;
            if (bytes > 1024 * 1024 || bytes % sizeof(AudioObjectID) != 0) return -2;
            NSMutableData *data = [NSMutableData dataWithLength:bytes];
            if (AudioObjectGetPropertyData(kAudioObjectSystemObject, &list, 0, NULL, &bytes, data.mutableBytes) != noErr) return -2;
            const AudioObjectID *processes = data.bytes;
            for (UInt32 index = 0; index < bytes / sizeof(AudioObjectID); index++) {
                AudioObjectPropertyAddress active = { kAudioProcessPropertyIsRunningOutput, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain };
                UInt32 running = 0, size = sizeof(running);
                if (AudioObjectGetPropertyData(processes[index], &active, 0, NULL, &size, &running) != noErr || !running) continue;
                AudioObjectPropertyAddress identifier = { kAudioProcessPropertyBundleID, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyElementMain };
                CFStringRef bundle = NULL; size = sizeof(bundle);
                if (AudioObjectGetPropertyData(processes[index], &identifier, 0, NULL, &size, &bundle) != noErr || !bundle) continue;
                BOOL browser = Cinna_IsBrowser((__bridge NSString *)bundle);
                CFRelease(bundle);
                if (browser) return 1;
            }
            return 0;
        }
        return -1;
    }
}
