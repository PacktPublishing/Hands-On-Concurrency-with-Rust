function (id, skipped_packets, dest_tbl, src_tbl, kind_tbl)
   print("ID: ", id)
   print("SKIPPED PACKETS: ", skipped_packets)
   print("DESTINATIONS:")
   for k,v in pairs(dest_tbl) do
      print(k, v)
   end
   print("SOURCES:")
   for k,v in pairs(src_tbl) do
      print(k, v)
   end
   print("KINDS:")
   for k,v in pairs(kind_tbl) do
      print(k, v)
   end
end
