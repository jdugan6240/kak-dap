import debug_session

from functools import partial

scopes = []
pending_var_requests = 0


class Expandable:
    def __init__(self):
        self.variable_reference = 0
        self.children = []
        self.contents = {}


def handle_scopes_response(msg):
    global scopes, pending_var_requests
    # TODO: Make scopes and variables persistent
    scopes.clear()

    scopes_members = msg["body"]["scopes"]
    for val in scopes_members:
        scope = Expandable()
        scope.variable_reference = int(val["variablesReference"])
        scope.contents = val
        scopes.append(scope)
        # Request variables for this scope
        var_ref = int(val["variablesReference"])
        var_args = {
            "variablesReference": var_ref
        }
        pending_var_requests += 1
        debug_session.debug_adapter.write_request(
            "variables", var_args, partial(handle_variables_response, parent=scope))


def handle_variables_response(msg, parent):
    global variables, pending_var_requests

    pending_var_requests -= 1

    # Loop over every variable in this response
    var_members = msg["body"]["variables"]
    for val in var_members:
        variable = Expandable()
        variable.variable_reference = int(val["variablesReference"])
        variable.contents = val
        parent.children.append(variable)

    # If we've serviced all pending variable requests, render the scopes and variables
    if pending_var_requests == 0:
        serialize_scopes()


def serialize_scopes():
    cmd = "dap-show-variables '"
    cmd_val = ""
    for scope in scopes:
        scope_name = scope.contents["name"]
        cmd_val += f"Scope: {scope_name}"
        cmd_val += "\n"
        # If this scope has child variables, render them
        if len(scope.children) != 0:
            cmd_val += serialize_variables(scope.children, 2)

    cmd += debug_session.kak_connection.escape_str(cmd_val)
    cmd += "'"
    debug_session.kak_connection.send_cmd(cmd)    


def serialize_variables(variables, indent):
    val = ""
    icon = " " # + if collapsed, - if expanded, ' ' otherwise
    for var in variables:
        # Indent
        for i in range(0, indent):
            val += " "
        # Determine proper icon
        if var.variable_reference > 0:
            icon = "+"
            if len(var.children) != 0:
                icon = "-"
        # Render variable
        var_name = var.contents["name"]
        var_type = var.contents["type"]
        var_value = var.contents["value"]
        val += f"{icon} "
        val += f"{var_name} ({var_type}): {var_value}\n"
        # If variable is expanded, render children
        if len(var.children) != 0:
            val += serialize_variables(var.children, indent + 2)
    return val
