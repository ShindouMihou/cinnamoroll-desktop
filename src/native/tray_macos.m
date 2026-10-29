#import <Cocoa/Cocoa.h>
#include <stdint.h>

typedef void (*CinnaCallback)(void *, void *);

@interface CinnaMenu : NSMenu
@property(nonatomic, strong) NSMutableArray *entries;
@end
@implementation CinnaMenu
- (instancetype)init {
    if ((self = [super initWithTitle:@""])) { self.entries = [NSMutableArray array]; self.autoenablesItems = NO; }
    return self;
}
@end

@interface CinnaEntry : NSObject
@property(nonatomic, strong) NSMenuItem *item;
@property(nonatomic, strong) CinnaMenu *submenu;
@property(nonatomic) CinnaCallback callback;
@property(nonatomic) void *context;
@property(nonatomic) uint32_t flags;
- (void)invoke:(id)sender;
@end
@implementation CinnaEntry
- (void)invoke:(id)sender {
    if (self.flags & 2) self.item.state = self.item.state == NSControlStateValueOn ? NSControlStateValueOff : NSControlStateValueOn;
    if (self.callback) self.callback(self.context, (__bridge void *)self);
}
@end

@interface CinnaTray : NSObject
@property(nonatomic, strong) NSStatusItem *item;
@property(nonatomic, strong) CinnaMenu *menu;
@end
@implementation CinnaTray
@end

void *Cinna_CreateTray(const uint8_t *pixels, int width, int height, int pitch, const char *tooltip) {
    if (![NSThread isMainThread]) return NULL;
    if (NSApp.activationPolicy == NSApplicationActivationPolicyProhibited) [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
    CinnaTray *tray = [CinnaTray new];
    tray.item = [[NSStatusBar systemStatusBar] statusItemWithLength:NSSquareStatusItemLength];
    if (!tray.item || !tray.item.button) return NULL;
    tray.item.autosaveName = @"CinnamorollDesktopMenu";
    tray.item.visible = YES;
    tray.item.button.toolTip = tooltip ? [NSString stringWithUTF8String:tooltip] : @"Cinnamoroll";
    tray.item.button.accessibilityLabel = @"Cinnamoroll";
    NSBitmapImageRep *bitmap = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:NULL pixelsWide:width pixelsHigh:height bitsPerSample:8 samplesPerPixel:4 hasAlpha:YES isPlanar:NO colorSpaceName:NSCalibratedRGBColorSpace bytesPerRow:width * 4 bitsPerPixel:32];
    if (!bitmap) { [[NSStatusBar systemStatusBar] removeStatusItem:tray.item]; return NULL; }
    for (int row = 0; row < height; row++) memcpy(bitmap.bitmapData + row * bitmap.bytesPerRow, pixels + row * pitch, width * 4);
    NSImage *image = [[NSImage alloc] initWithSize:NSMakeSize(22, 22)];
    [image addRepresentation:bitmap];
    tray.item.button.image = image;
    return (__bridge_retained void *)tray;
}

void *Cinna_CreateMenu(void *raw) {
    CinnaTray *tray = (__bridge CinnaTray *)raw;
    tray.menu = [CinnaMenu new];
    tray.item.menu = tray.menu;
    return (__bridge void *)tray.menu;
}

void *Cinna_CreateSubmenu(void *raw) {
    CinnaEntry *entry = (__bridge CinnaEntry *)raw;
    entry.submenu = [CinnaMenu new];
    entry.item.submenu = entry.submenu;
    return (__bridge void *)entry.submenu;
}

void *Cinna_InsertEntry(void *raw, int position, const char *label, uint32_t flags) {
    CinnaMenu *menu = (__bridge CinnaMenu *)raw;
    CinnaEntry *entry = [CinnaEntry new];
    entry.flags = flags;
    if (label) {
        entry.item = [[NSMenuItem alloc] initWithTitle:[NSString stringWithUTF8String:label] action:@selector(invoke:) keyEquivalent:@""];
        entry.item.target = entry;
        entry.item.enabled = !(flags & 0x80000000u);
        entry.item.state = flags & 0x40000000u ? NSControlStateValueOn : NSControlStateValueOff;
    } else entry.item = [NSMenuItem separatorItem];
    [menu.entries addObject:entry];
    NSInteger index = position < 0 ? menu.numberOfItems : MIN(position, menu.numberOfItems);
    [menu insertItem:entry.item atIndex:index];
    return (__bridge void *)entry;
}

void Cinna_SetCallback(void *raw, CinnaCallback callback, void *context) {
    CinnaEntry *entry = (__bridge CinnaEntry *)raw;
    entry.callback = callback; entry.context = context;
}
void Cinna_SetLabel(void *raw, const char *label) { ((__bridge CinnaEntry *)raw).item.title = [NSString stringWithUTF8String:label]; }
void Cinna_SetChecked(void *raw, bool checked) { ((__bridge CinnaEntry *)raw).item.state = checked ? NSControlStateValueOn : NSControlStateValueOff; }
bool Cinna_TrayVisible(void *raw) {
    CinnaTray *tray = (__bridge CinnaTray *)raw;
    NSWindow *window = tray.item.button.window;
    return tray.item.visible && tray.item.button.image.valid && window.visible && window.frame.origin.y >= NSMaxY(window.screen.frame) - 64.;
}
void Cinna_DestroyTray(void *raw) {
    CinnaTray *tray = (__bridge_transfer CinnaTray *)raw;
    [[NSStatusBar systemStatusBar] removeStatusItem:tray.item];
    tray.item.menu = nil;
}
const char *Cinna_TrayDescription(void *raw) {
    CinnaTray *tray = (__bridge CinnaTray *)raw;
    NSWindow *window = tray.item.button.window;
    NSString *description = [NSString stringWithFormat:@"bundle=%@ policy=%ld item=%d window=%d frame=%@ screen=%@ screens=%lu pid=%d", NSBundle.mainBundle.bundleIdentifier, (long)NSApp.activationPolicy, tray.item.visible, window.visible, NSStringFromRect(window.frame), NSStringFromRect(window.screen.frame), (unsigned long)NSScreen.screens.count, NSProcessInfo.processInfo.processIdentifier];
    return description.UTF8String;
}
void Cinna_BootstrapTray(void) {
    if (NSApp.running) return;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, NSEC_PER_SEC / 4), dispatch_get_main_queue(), ^{
        [NSApp stop:nil];
        NSEvent *event = [NSEvent otherEventWithType:NSEventTypeApplicationDefined location:NSZeroPoint modifierFlags:0 timestamp:0 windowNumber:0 context:nil subtype:0 data1:0 data2:0];
        [NSApp postEvent:event atStart:NO];
    });
    [NSApp run];
}
