# Parses all new, modified or added toml dialog files and creates a png graphs from them.
# Visualization of the dialog tree is a vital part of the story making process.

from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer
from pathlib import Path
import csv
import graphviz
import hashlib
import os
import sys
import time
import toml


ASSET_DIR_PATH = "main_game/assets/dialogs"
OUTPUT_DIR_PATH = ".devtools/dialog_visualizations"
CACHE_FILE_PATH = f"{OUTPUT_DIR_PATH}/cache.csv"


def visualize_dialog(dialog_toml):
    graph = graphviz.Digraph(
        format="png",
        graph_attr={"rankdir": "LR", "pad": "1.0", "nodesep": "1.0", "ranksep": "1.0"},
    )

    # Parse TOML
    dialog_data = toml.loads(dialog_toml)

    try:
        root = dialog_data["root"]

        if "name" not in root:
            # default root name
            root["name"] = "_root"
        elif root["name"] != "_root":
            # if provided, name must always equal to _root
            raise ValueError("Root object name should be '_root'")

        # node list is now optional, so default it to a list
        if "node" not in dialog_data:
            dialog_data["node"] = []

        # add root object to the node list
        dialog_data["node"].insert(0, root)
    except KeyError:
        raise KeyError("Root object is not present in dialog_data")

    # Add nodes to the graph
    for index, node in enumerate(dialog_data["node"]):
        node_name = node.get("name", f"node_{index}")

        # default styles
        shape = "note"
        style = None
        fillcolor = None
        fontname = None

        # string array of rows, each item will be prepended with <TR> and
        # appended with </TR> eventually
        rows = []

        if "name" in node:
            rows.append(f"<TD><B>{node_name}</B></TD>")

        if "guard" in node:
            shape = "component"
            style = "filled"
            fillcolor = "lightblue"
            fontname = "Monospace matrix=1 .1 0 1"

            rows.append("<TD>{}</TD>".format(node["guard"]))
            if "params" in node:
                rows.append("<TD>{}</TD>".format(node["params"]))

        if "who" in node:
            # with respect to the script path
            image_path = "main_game/assets/characters/portraits/"
            match node["who"]:
                case "Redhead":
                    image_path += "redhead1.png"
                case "Winnie":
                    image_path += "winnie1.png"
                case "Marie":
                    image_path += "marie1.png"
                case "Bolt":
                    image_path += "bolt1.png"
                case _:
                    exit(f"Character {node['who']} not ready for visualization.")

            if not os.path.exists(image_path):
                raise ValueError(f"Image {image_path} not found.")

            rows.append(
                f"""<TD FIXEDSIZE="true" width="64" height="64"><IMG SRC="../../{image_path}" /></TD>"""
            )

        if "en" in node:
            rows.append("<TD>{}</TD>".format(node["en"].replace("\n", "<BR/>")))

        # for each row in rows, prepend with <TR> and append with </TR> and then join them
        rows = "".join([f"<TR>{row}</TR>" for row in rows])
        graph.node(
            node_name,
            label=f"""<<TABLE CELLSPACING="2" CELLPADDING="2" BORDER="0">{rows}</TABLE>>""",
            shape=shape,
            style=style,
            fillcolor=fillcolor,
            fontname=fontname,
        )

    # Add edges to the graph
    for index, node in enumerate(dialog_data["node"]):
        node_name = node.get("name", f"node_{index}")
        if "next" in node:
            # prints whether next is a string of array of strings
            next_nodes = (
                node["next"] if isinstance(node["next"], list) else [node["next"]]
            )
            nodes_len = len(next_nodes)
            for node_index, next_node in enumerate(next_nodes):

                match next_node:
                    case "_emerge":
                        exit_name = f"_emerge_{index}"
                        graph.node(
                            exit_name,
                            fillcolor="lightgreen",
                            shape="triangle",
                            style="filled",
                            label="",
                        )
                        graph.edge(node_name, exit_name)
                    case "_end_dialog":
                        exit_name = f"_end_dialog_{index}"
                        graph.node(
                            exit_name,
                            fillcolor="red",
                            shape="triangle",
                            style="filled",
                            label="",
                        )
                        graph.edge(node_name, exit_name)
                    case _:
                        graph.edge(
                            node_name,
                            next_node,
                            label=f"{node_index + 1}" if nodes_len > 1 else None,
                        )
        else:
            # If node doesn't have 'next', connect to the next node in the array
            next_index = index + 1
            # if out of bounds (this is the last node) then crash
            if next_index >= len(dialog_data["node"]):
                raise ValueError(
                    f"Node {node_name} has no 'next' and is the last node in the dialog"
                )

            next_node_name = dialog_data["node"][next_index].get(
                "name", f"node_{next_index}"
            )
            graph.edge(node_name, next_node_name)

    return graph


def compute_hash(file):
    hasher = hashlib.sha256()
    hasher.update(file.encode())
    return hasher.hexdigest()


def list_toml_files(directory):
    toml_files = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith(".toml"):
                toml_files.append(os.path.join(root, file))

    return toml_files


def load_cache():
    if os.path.isfile(CACHE_FILE_PATH):
        with open(CACHE_FILE_PATH, "r", newline="") as file:
            reader = csv.reader(file)
            cache = {row[0]: row[1] for row in reader}
    else:
        cache = {}

    return cache


def store_cache():
    with open(CACHE_FILE_PATH, "w", newline="") as file:
        writer = csv.writer(file)
        for file_path, file_hash in cache.items():
            writer.writerow([file_path, file_hash])


def render_dialog(file_path, dialog_toml):
    print(f"Visualizing {file_path}")

    graph = visualize_dialog(dialog_toml)
    file_stem = Path(file_path).stem
    graph.render(filename=f"{OUTPUT_DIR_PATH}/{file_stem}", format="png", cleanup=True)


def catch_up_visualization(cache):
    toml_files = list_toml_files(ASSET_DIR_PATH)

    # Iterate through .toml files
    for file_path in toml_files:
        with open(file_path, "r") as file:
            dialog_toml = file.read()

        file_hash = compute_hash(dialog_toml)

        # Check if the file path is in the cache and if the hash has changed
        if file_path not in cache or cache[file_path] != file_hash:
            cache[file_path] = file_hash
            render_dialog(file_path, dialog_toml)

    store_cache()


class FileModifiedHandler(FileSystemEventHandler):
    def __init__(self, cache):
        self.cache = cache
        self.needs_save = False

    def on_modified(self, event):
        file_path = event.src_path

        if event.is_directory or not file_path.endswith(".toml"):
            return

        with open(file_path, "r") as file:
            dialog_toml = file.read()

        # try render dialog, don't do anything if it fails
        try:
            render_dialog(file_path, dialog_toml)
            file_hash = compute_hash(dialog_toml)
            self.cache[file_path] = file_hash
            self.needs_save = True
        except Exception as e:
            print(f"Failed to render {file_path}: {e}")


def watch_for_changes(cache):
    # Instantiate a file system event handler
    event_handler = FileModifiedHandler(cache)

    # Create an observer to watch for file system events
    observer = Observer()
    observer.schedule(event_handler, ASSET_DIR_PATH, recursive=True)
    observer.start()

    try:
        print("Watching for changes...")
        while True:
            time.sleep(5)
            if event_handler.needs_save:
                store_cache()
                event_handler.needs_save = False

    except KeyboardInterrupt:
        observer.stop()
    observer.join()


if __name__ == "__main__":
    cache = load_cache()
    catch_up_visualization(cache)

    # if --watch was provided, start watching for changes
    if len(sys.argv) > 1 and sys.argv[1] == "--watch":
        watch_for_changes(cache)
