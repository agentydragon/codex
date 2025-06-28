+++
id = "51"
title = "Add event layer for widget consolidation"
status = "open"
freeform_status = ""
dependencies = []
last_updated = "2025-06-26T06:30:00Z"
+++

# Task 51: Add event layer for widget consolidation

Introduce a new abstraction layer between raw events and rendered conversation screen widgets. This layer will initially pass through all events unchanged, and over time enable grouping and collapsing related events (e.g. command start/end) into single UI cells for cleaner command status display.
