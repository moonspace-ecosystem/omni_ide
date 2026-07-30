import sys
import json
import Quartz
from AppKit import NSWorkspace

def get_zed_pid():
    workspace = NSWorkspace.sharedWorkspace()
    for app in workspace.runningApplications():
        name = app.localizedName()
        if name == "Zed" or name == "Omni IDE" or name == "zed":
            return app.processIdentifier()
    return None

def dump_element(element, depth=0):
    if not element:
        return None
    try:
        role = element.attributeValue_("AXRole")
        title = element.attributeValue_("AXTitle")
        val = element.attributeValue_("AXValue")
        desc = element.attributeValue_("AXDescription")
        
        node = {"role": role, "title": title, "value": val, "desc": desc}
        children_ref = element.attributeValue_("AXChildren")
        if children_ref:
            node["children"] = []
            for child in children_ref:
                child_node = dump_element(child, depth + 1)
                if child_node:
                    node["children"].append(child_node)
        return node
    except Exception as e:
        return {"error": str(e)}

pid = get_zed_pid()
if not pid:
    print(json.dumps({"error": "Omni IDE / Zed is not running"}))
    sys.exit(1)

app_element = Quartz.AXUIElementCreateApplication(pid)
tree = dump_element(app_element)
print(json.dumps(tree, indent=2))
