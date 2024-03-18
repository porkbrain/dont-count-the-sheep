# Parses toml file and creates a png graph from it.
# Visualization of the dialog tree is a vital part of the story process.

import graphviz
import toml
import os


def visualize_dialog(dialog_toml):
    graph = graphviz.Digraph(
        format="png",
        graph_attr={"rankdir": "LR", "pad": "1.0", "nodesep": "1.0", "ranksep": "1.0"},
    )

    # Parse TOML
    dialog_data = toml.loads(dialog_toml)

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
            image_path = "/home/porkbrain/Code/dont-count-the-sheep/main_game/assets/characters/portraits/"
            match node["who"]:
                case "Redhead":
                    image_path += "redhead1.png"
                case "Winnie":
                    image_path += "winnie1.png"
                case _:
                    exit(f"Character {node['who']} not found.")

            if not os.path.exists(image_path):
                raise ValueError(f"Image {image_path} not found.")

            rows.append(
                f"""<TD FIXEDSIZE="true" width="64" height="64"><IMG SRC="{image_path}" /></TD>"""
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
            nodes_len = len(node["next"])
            for node_index, next_node in enumerate(node["next"]):

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
                        graph.edge(node_name, next_node, label=f"{node_index + 1}" if nodes_len > 1 else None)
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


if __name__ == "__main__":
    # Load TOML from file
    with open("test.toml", "r") as file:
        dialog_toml = file.read()

    graph = visualize_dialog(dialog_toml)
    # print graph
    print(graph.source)
    # and now into image
    graph.render(filename="dialog_graph", format="png", cleanup=True, view=True)
