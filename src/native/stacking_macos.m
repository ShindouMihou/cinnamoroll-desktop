#import <AppKit/AppKit.h>

int CinnaWindowStackState(void *lowerPointer, void *upperPointer) {
    NSWindow *lower = (__bridge NSWindow *)lowerPointer;
    NSWindow *upper = (__bridge NSWindow *)upperPointer;
    if (!lower || !upper) return -1;
    if (!lower.visible || !upper.visible) return 0;
    NSArray<NSWindow *> *windows = NSApp.orderedWindows;
    NSUInteger lowerIndex = [windows indexOfObjectIdenticalTo:lower];
    NSUInteger upperIndex = [windows indexOfObjectIdenticalTo:upper];
    if (lowerIndex == NSNotFound || upperIndex == NSNotFound) return 0;
    return lowerIndex < upperIndex ? 1 : 0;
}

void CinnaWindowOrderBelow(void *lowerPointer, void *upperPointer) {
    NSWindow *lower = (__bridge NSWindow *)lowerPointer;
    NSWindow *upper = (__bridge NSWindow *)upperPointer;
    if (lower && upper && lower.visible && upper.visible) {
        lower.level = upper.level;
        [lower orderWindow:NSWindowBelow relativeTo:upper.windowNumber];
    }
}
