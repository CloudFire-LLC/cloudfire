/* Licensed under Apache 2.0 (C) 2023 Firezone, Inc. */
package dev.firezone.android.tunnel.util

import android.content.Context
import android.net.ConnectivityManager
import android.net.LinkProperties
import android.os.Build
import android.util.Log
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.LineNumberReader
import java.net.InetAddress

/**
 * DNS servers detector
 *
 * IMPORTANT: don't cache the result.
 *
 * Or if you want to cache the result make sure you invalidate the cache
 * on any network change.
 *
 * It is always better to use a new instance of the detector when you need
 * current DNS servers otherwise you may get into trouble because of invalid/changed
 * DNS servers.
 *
 * This class combines various methods and solutions from:
 * Dnsjava http://www.xbill.org/dnsjava/
 * Minidns https://github.com/MiniDNS/minidns
 * https://stackoverflow.com/a/48973823/1275497
 *
 * Unfortunately both libraries are not aware of Oreo changes so new method was added to fix this.
 *
 * Created by Madalin Grigore-Enescu on 2/24/18.
 */
class DnsServersDetector(
    /**
     * Holds context this was created under
     */
    private val context: Context,
) {
    //region - public //////////////////////////////////////////////////////////////////////////////
    // //////////////////////////////////////////////////////////////////////////////////////////////
    val servers: Array<String>
        /**
         * Returns android DNS servers used for current connected network
         * @return Dns servers array
         */
        get() {
            // use connectivity manager
            serversMethodConnectivityManager?.run {
                if (isNotEmpty()) {
                    return this
                }
            }

            // detect android DNS servers by executing getprop string command in a separate process
            // This method fortunately works in Oreo too but many people may want to avoid exec
            // so it's used only as a failsafe scenario
            serversMethodExec?.run {
                if (isNotEmpty()) {
                    return this
                }
            }

            // Fall back on factory DNS servers
            return FACTORY_DNS_SERVERS
        }

    //endregion
    //region - private /////////////////////////////////////////////////////////////////////////////
    // //////////////////////////////////////////////////////////////////////////////////////////////
    private val serversMethodConnectivityManager: Array<String>?
        /**
         * Detect android DNS servers by using connectivity manager
         *
         * This method is working in android LOLLIPOP or later
         *
         * @return Dns servers array
         */
        get() {
            // This code only works on LOLLIPOP and higher
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                try {
                    val priorityServersArrayList = ArrayList<String>()
                    val serversArrayList = ArrayList<String>()
                    val connectivityManager =
                        context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager?
                    if (connectivityManager != null) {

                        // Iterate all networks
                        // Notice that android LOLLIPOP or higher allow iterating multiple connected networks of SAME type
                        for (network in connectivityManager.allNetworks) {
                            val networkInfo = connectivityManager.getNetworkInfo(network)
                            if (networkInfo!!.isConnected) {
                                val linkProperties = connectivityManager.getLinkProperties(network)
                                val dnsServersList = linkProperties!!.dnsServers

                                // Prioritize the DNS servers for link which have a default route
                                if (linkPropertiesHasDefaultRoute(linkProperties)) {
                                    for (element in dnsServersList) {
                                        val dnsHost = element.hostAddress
                                        dnsHost?.let {
                                            priorityServersArrayList.add(it)
                                        }
                                    }
                                } else {
                                    for (element in dnsServersList) {
                                        val dnsHost = element.hostAddress
                                        dnsHost?.let {
                                            serversArrayList.add(it)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Append secondary arrays only if priority is empty
                    if (priorityServersArrayList.isEmpty()) {
                        priorityServersArrayList.addAll(serversArrayList)
                    }

                    // Stop here if we have at least one DNS server
                    if (priorityServersArrayList.size > 0) {
                        return priorityServersArrayList.toTypedArray()
                    }
                } catch (ex: Exception) {
                    Log.d(
                        TAG,
                        "Exception detecting DNS servers using ConnectivityManager method",
                        ex,
                    )
                }
            }

            // Failure
            return null
        }
    private val serversMethodExec: Array<String>?
        /**
         * Detect android DNS servers by executing getprop string command in a separate process
         *
         * Notice there is an android bug when Runtime.exec() hangs without providing a Process object.
         * This problem is fixed in Jelly Bean (Android 4.1) but not in ICS (4.0.4) and probably it will never be fixed in ICS.
         * https://stackoverflow.com/questions/8688382/runtime-exec-bug-hangs-without-providing-a-process-object/11362081
         *
         * @return Dns servers array
         */
        get() {
            // We are on the safe side and avoid any bug
            try {
                val process = Runtime.getRuntime().exec("getprop")
                val inputStream = process.inputStream
                val lineNumberReader = LineNumberReader(InputStreamReader(inputStream))
                val serversSet = methodExecParseProps(lineNumberReader)
                if (serversSet.isNotEmpty()) {
                    return serversSet.toTypedArray()
                }
            } catch (ex: Exception) {
                Log.d(TAG, "Exception in getServersMethodExec", ex)
            }

            // Failed
            return null
        }

    /**
     * Parse properties produced by executing getprop command
     * @param lineNumberReader
     * @return Set of parsed properties
     * @throws Exception
     */
    @Throws(Exception::class)
    private fun methodExecParseProps(lineNumberReader: BufferedReader): Set<String> {
        var line: String
        val serversSet: MutableSet<String> = HashSet(10)
        while (lineNumberReader.readLine().also { line = it } != null) {
            val split = line.indexOf(METHOD_EXEC_PROP_DELIM)
            if (split == -1) {
                continue
            }
            val property = line.substring(1, split)
            val valueStart = split + METHOD_EXEC_PROP_DELIM.length
            val valueEnd = line.length - 1
            if (valueEnd < valueStart) {
                // This can happen if a newline sneaks in as the first character of the property value. For example
                // "[propName]: [\n…]".
                Log.d(TAG, "Malformed property detected: \"$line\"")
                continue
            }
            val value = line.substring(valueStart, valueEnd)
            if (value.isEmpty()) {
                continue
            }
            if (property.endsWith(".dns") || property.endsWith(".dns1") ||
                property.endsWith(".dns2") || property.endsWith(".dns3") ||
                property.endsWith(".dns4")
            ) {
                InetAddress.getByName(value).hostAddress?.takeIf { it.isNotEmpty() }?.let {
                    serversSet.add(it)
                }
            }
        }
        return serversSet
    }

    /**
     * Returns true if the specified link properties have any default route
     * @param linkProperties
     * @return true if the specified link properties have default route or false otherwise
     */
    private fun linkPropertiesHasDefaultRoute(linkProperties: LinkProperties?): Boolean {
        for (route in linkProperties!!.routes) {
            if (route.isDefaultRoute) {
                return true
            }
        }
        return false
    } //endregion

    companion object {
        private const val TAG = "DnsServersDetector"

        /**
         * Holds some default DNS servers used in case all DNS servers detection methods fail.
         * Can be set to null if you want caller to fail in this situation.
         */
        private val FACTORY_DNS_SERVERS = arrayOf(
            "8.8.8.8",
            "8.8.4.4",
        )

        /**
         * Properties delimiter used in exec method of DNS servers detection
         */
        private const val METHOD_EXEC_PROP_DELIM = "]: ["
    }
}
